use axum::{Json, http::StatusCode};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::auth::LoginRequest;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,   // user id
    pub exp: usize, // expiration timestamp
}

#[derive(Serialize)]
pub enum LoginResponse {
    #[serde(rename = "token")]
    Token(String),
    #[serde(rename = "error")]
    Error(String),
}

pub async fn login(
    db: axum::extract::Extension<SqlitePool>,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<LoginResponse>) {
    let user = match sqlx::query!(
        "SELECT id, password_hash FROM users WHERE username = ?",
        payload.username
    )
    .fetch_one(&db.0)
    .await
    {
        Ok(user) => user,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(LoginResponse::Error("User does not exist".to_string())),
            );
        }
    };

    match crate::auth::validate(&payload.username, &payload.password, db).await {
        Ok(true) => {}
        Ok(false) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(LoginResponse::Error("Invalid credentials".to_string())),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginResponse::Error(format!("Error validating user"))),
            );
        }
    };

    let claims = Claims {
        sub: user.id,
        exp: (chrono::Utc::now().timestamp() + 3600) as usize,
    };

    let secret = match std::env::var("JWT_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            tracing::error!("JWT_SECRET environment variable not set");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginResponse::Error(
                    "Server configuration error".to_string(),
                )),
            );
        }
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Failed to create JWT token: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginResponse::Error("Failed to create token".to_string())),
            );
        }
    };

    (StatusCode::OK, Json(LoginResponse::Token(token)))
}
