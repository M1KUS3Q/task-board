use axum::{Json, http::StatusCode};
use axum_extra::{
    TypedHeader,
    headers::{self, Authorization},
};
use sqlx::SqlitePool;

use super::utils::map_auth_error;
use crate::{
    auth::utils,
    boards::{
        api::utils::ensure_minimum_role, board::BoardId, role::BoardUserRole, utils::TraceError,
    },
};

#[derive(serde::Deserialize)]
pub struct BoardCreateRequest {
    pub name: String,
    pub description: Option<String>,
}

pub async fn create_board(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    Json(payload): Json<BoardCreateRequest>,
) -> Result<Json<BoardId>, (StatusCode, String)> {
    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let board_id = crate::boards::board::Board::create_board(&db.0, user_id, payload.name)
        .await
        .trace_err("Failed to create board")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    if let Some(desc) = payload.description {
        crate::boards::board::Board::update_board_description(&db.0, board_id, desc)
            .await
            .trace_err("Failed to set board description")
            .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;
    }

    Ok(Json(board_id))
}

#[derive(serde::Deserialize)]
pub struct BoardDeleteRequest {
    pub board_id: BoardId,
}
pub async fn delete_board(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    Json(payload): Json<BoardDeleteRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    // Verify user is owner of the board
    let role = BoardUserRole::get_user_role_on_board(&db.0, payload.board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    ensure_minimum_role(
        role,
        BoardUserRole::Owner,
        "Only board owners can delete the board",
    )?;

    Ok(StatusCode::OK)
}

#[derive(serde::Deserialize)]
pub struct BoardUpdateRequest {
    pub board_id: BoardId,
    pub name: Option<String>,
    pub description: Option<String>,
}
pub async fn update_board_details(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    Json(payload): Json<BoardUpdateRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if payload.name.is_none() && payload.description.is_none() {
        return Err((StatusCode::BAD_REQUEST, "No fields to update".to_string()));
    }

    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    // Verify user is owner of the board
    let role = BoardUserRole::get_user_role_on_board(&db.0, payload.board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;
    ensure_minimum_role(
        role,
        BoardUserRole::Owner,
        "Only board owners can update the board",
    )?;

    if let Some(name) = payload.name {
        crate::boards::board::Board::update_board_name(&db.0, payload.board_id, name)
            .await
            .trace_err("Failed to update board name")
            .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;
    }

    if let Some(desc) = payload.description {
        crate::boards::board::Board::update_board_description(&db.0, payload.board_id, desc)
            .await
            .trace_err("Failed to update board description")
            .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;
    }

    Ok(StatusCode::OK)
}

#[derive(serde::Serialize)]
pub struct BoardDetails {
    pub name: String,
    pub description: Option<String>,
}

pub async fn get_board_details(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    axum::extract::Path(board_id): axum::extract::Path<BoardId>,
) -> Result<Json<BoardDetails>, (StatusCode, String)> {
    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    // Verify user has access to the board
    let role = BoardUserRole::get_user_role_on_board(&db.0, board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;
    ensure_minimum_role(role, BoardUserRole::Viewer, "No access to this board")?;

    let board = crate::boards::board::Board::get_board(&db.0, board_id)
        .await
        .trace_err("Failed to fetch board details")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?
        .ok_or((StatusCode::NOT_FOUND, "Board not found".to_string()))?;

    Ok(Json(BoardDetails {
        name: board.name,
        description: board.description,
    }))
}
