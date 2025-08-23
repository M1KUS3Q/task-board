use axum::{Json, http::StatusCode};
use axum_extra::{
    TypedHeader,
    headers::{self, Authorization},
};
use jsonwebtoken::{DecodingKey, Validation};
use sqlx::SqlitePool;

use crate::auth::login::Claims;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MeResponse {
    id: i64,
    username: String,
}

pub async fn me(
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    db: axum::extract::Extension<SqlitePool>,
) -> Result<Json<MeResponse>, StatusCode> {
    let token_data = jsonwebtoken::decode::<Claims>(
        auth.token(),
        &DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_ref()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if token_data.claims.exp < chrono::Utc::now().timestamp() as usize {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user_id = token_data.claims.sub;
    let username = sqlx::query_scalar!("SELECT username FROM users WHERE id = ?", user_id)
        .fetch_one(&db.0)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(MeResponse {
        id: user_id,
        username,
    }))
}
