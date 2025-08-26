use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::info;

/// LoginRequest holds the user's credentials for authentication.
///
/// # Fields
///
/// * `username` - A unique identifier for the user. Must be a non-empty string.
/// * `password` - The user's plaintext password. This will be compared against the stored hash in the database.

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Checks if a given username already exists in the database.
///
/// Executes a SQL query to count matching usernames in the `users` table.
///
/// # Arguments
///
/// * `username` - The username to check for existence.
/// * `db` - The SQLite connection pool extracted by Axum.
///
/// # Returns
///
/// Returns `Ok(true)` if the username is found, `Ok(false)` otherwise.
/// Returns `Err(String)` with a descriptive error message if the database operation fails.
///
/// # Errors
///
/// Returns `Err("Failed to check existing username")` if the query or fetch fails.
///
/// # Examples
///
/// ```rust,no_run
/// let exists = username_exists("alice", db).await?;
/// if exists {
///     println!("Username is already taken");
/// }
/// ```
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

/// Hashes a plaintext password using Argon2 with a randomly generated salt.
///
/// Generates a 16-byte random salt and encodes the password using the default Argon2 configuration.
///
/// # Arguments
///
/// * `password` - The plaintext password to hash.
///
/// # Returns
///
/// Returns `Ok(String)` containing the encoded hash if successful.
/// Returns `Err(String)` with an error message if hashing fails.
///
/// # Errors
///
/// Returns `Err("Failed to hash password")` if Argon2 encoding encounters an error.
///
/// # Examples
///
/// ```rust
/// let password_hash = hash_password("s3cr3tP@ssw0rd")?;
/// ```
pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = rand::random::<[u8; 16]>();
    argon2::hash_encoded(password.as_bytes(), &salt, &argon2::Config::default())
        .map_err(|_| "Failed to hash password".to_string())
}

/// Validates a user's credentials by comparing the provided password against the stored hash.
///
/// Fetches the password hash for the given username and uses Argon2 to verify the provided password.
///
/// # Arguments
///
/// * `username` - The username whose credentials are being validated. Must not be empty.
/// * `password` - The plaintext password to verify. Must not be empty.
/// * `db` - The SQLite connection pool extracted by Axum.
///
/// # Returns
///
/// Returns `Ok(true)` if the username exists and the password matches the stored hash.
/// Returns `Ok(false)` if the username exists but the password does not match.
/// Returns `Err(String)` if the user is not found or if any database or verification error occurs.
///
/// # Panics
///
/// Panics if `username` or `password` are empty to enforce non-empty credential fields.
///
/// # Errors
///
/// Returns:
/// - `Err("Failed to validate user")` if the database query fails.
/// - `Err("User not found")` if no record exists for the provided username.
/// - `Err("Failed to verify password")` if Argon2 verification fails.
///
/// # Examples
///
/// ```rust,no_run
/// let is_valid = validate("alice", "correct-horse-battery-staple", db).await?;
/// if is_valid {
///     println!("Authentication successful");
/// } else {
///     println!("Invalid credentials");
/// }
/// ```
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
