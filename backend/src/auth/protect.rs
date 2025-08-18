use axum::{Json, http::StatusCode};
use axum_extra::{
    TypedHeader,
    headers::{self, Authorization},
};
use jsonwebtoken::{DecodingKey, Validation};
use sqlx::SqlitePool;

use crate::auth::login::Claims;

pub async fn protected_route(
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    db: axum::extract::Extension<SqlitePool>,
) -> Result<Json<String>, StatusCode> {
    let _ = db;
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

    // Here you can fetch user data or perform actions based on the user_id
    // For demonstration, we will just return a success message
    Ok(Json(format!(
        "Protected route accessed by user ID: {user_id}"
    )))
}
