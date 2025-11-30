use axum::http::StatusCode;

use crate::{auth::utils::AuthError, boards::role::BoardUserRole};

const NO_BOARD_ACCESS: &str = "No access to this board";

pub fn map_auth_error(err: AuthError) -> (StatusCode, String) {
    match err {
        AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),
        AuthError::ExpiredToken => (StatusCode::UNAUTHORIZED, "Expired token".to_string()),
    }
}

pub fn ensure_minimum_role(
    role: Option<BoardUserRole>,
    minimum_role: BoardUserRole,
    forbidden_msg: &str,
) -> Result<(), (StatusCode, String)> {
    match role {
        Some(r) if r.is_at_least(minimum_role) => Ok(()),
        Some(_) => Err((StatusCode::FORBIDDEN, forbidden_msg.to_string())),
        None => Err((StatusCode::FORBIDDEN, NO_BOARD_ACCESS.to_string())),
    }
}
