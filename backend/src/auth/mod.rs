use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::info;

pub mod login;
pub mod me;
pub mod protect;
pub mod signup;

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn username_exists(
    username: &str,
    db: axum::extract::Extension<SqlitePool>,
) -> Result<bool, String> {
    let exists = sqlx::query_scalar!("SELECT COUNT(*) FROM users WHERE username = ?", username)
        .fetch_one(&db.0)
        .await
        .map_err(|_| "Failed to check existing username")?;

    info!("Username {} exists: {:?}", username, exists);

    Ok(exists > 0)
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = rand::random::<[u8; 16]>();
    argon2::hash_encoded(password.as_bytes(), &salt, &argon2::Config::default())
        .map_err(|_| "Failed to hash password".to_string())
}

pub async fn validate(
    username: &str,
    password: &str,
    db: axum::extract::Extension<SqlitePool>,
) -> Result<bool, String> {
    assert!(!username.is_empty(), "Username cannot be empty");
    assert!(!password.is_empty(), "Password cannot be empty");

    let hash = sqlx::query!(
        "SELECT password_hash FROM users WHERE username = ?",
        username
    )
    .fetch_optional(&db.0)
    .await
    .map_err(|_| "Failed to validate user")?;

    info!("Validating user: {}, got record: {hash:?}", username);

    let Some(hash) = hash else {
        return Err("User not found".to_string());
    };

    Ok(
        argon2::verify_encoded(&hash.password_hash, password.as_bytes())
            .map_err(|_| "Failed to verify password")?,
    )
}
