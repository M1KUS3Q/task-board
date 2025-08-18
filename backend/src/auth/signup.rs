use axum::{Extension, Json, http::StatusCode};
use sqlx::SqlitePool;
use tracing::{error, info};

use crate::auth::{LoginRequest, hash_password, username_exists};

pub async fn signup(
    db: Extension<SqlitePool>,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<String>) {
    if username_exists(&payload.username, db.clone())
        .await
        .unwrap_or_else(|e| {
            error!("Failed to check username existence: {}", e);
            true
        })
    {
        return (
            StatusCode::CONFLICT,
            Json("Username already exists".to_string()),
        );
    }

    let hash = match hash_password(&payload.password) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash password: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Failed to hash password".to_string()),
            );
        }
    };

    match sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES (?, ?)",
        payload.username,
        hash
    )
    .execute(&db.0)
    .await
    {
        Ok(_) => {
            info!("User {} created successfully", payload.username);
            (
                StatusCode::CREATED,
                Json("User created successfully".to_string()),
            )
        }
        Err(e) => {
            error!("Failed to create user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Failed to create user".to_string()),
            )
        }
    }
}
