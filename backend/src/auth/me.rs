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

/// Retrieves the currently authenticated user's information.
///
/// This handler extracts the `Authorization: Bearer <token>` header,
/// decodes and validates the JWT against the `JWT_SECRET` environment variable,
/// checks for token expiration, and then queries the SQLite database
/// for the username corresponding to the user ID in the token claims.
///
/// # Parameters
///
/// * `auth` – a typed header extractor for `Authorization<Bearer>`, containing the JWT.
/// * `db` – an `Extension<SqlitePool>` for executing database queries.
///
/// # Returns
///
/// Returns `Ok(Json(MeResponse))` on success, containing:
/// - `id`: the user's ID extracted from the token subject (`sub`)
/// - `username`: the username fetched from the database
///
/// # Errors
///
/// * `StatusCode::UNAUTHORIZED` if the token is missing, invalid, or expired.
/// * `StatusCode::INTERNAL_SERVER_ERROR` if the database query fails.
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
