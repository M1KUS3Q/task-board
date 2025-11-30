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
        api::utils::ensure_minimum_role,
        board::BoardId,
        role::BoardUserRole,
        users::{BoardUser, UserId},
        utils::TraceError,
    },
};

#[derive(serde::Serialize)]
pub struct BoardUserResponse {
    pub board_id: BoardId,
    pub user_id: UserId,
    pub role: BoardUserRole,
}

pub async fn list_users_on_board(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    axum::extract::Path(board_id): axum::extract::Path<BoardId>,
) -> Result<Json<Vec<BoardUserResponse>>, (StatusCode, String)> {
    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let role = BoardUserRole::get_user_role_on_board(&db.0, board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    ensure_minimum_role(role, BoardUserRole::Viewer, "No access to this board")?;

    let users = BoardUser::get_users_on_board(&db.0, board_id)
        .await
        .trace_err("Failed to fetch board users")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?
        .into_iter()
        .map(|user| BoardUserResponse {
            board_id: user.board_id,
            user_id: user.user_id,
            role: user.role,
        })
        .collect();

    Ok(Json(users))
}

#[derive(serde::Deserialize)]
pub struct AddUserRequest {
    pub board_id: BoardId,
    pub user_id: UserId,
    pub role: BoardUserRole,
}

pub async fn add_user_to_board(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    Json(payload): Json<AddUserRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let role = BoardUserRole::get_user_role_on_board(&db.0, payload.board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    ensure_minimum_role(
        role,
        BoardUserRole::Owner,
        "Only board owners can manage board members",
    )?;

    BoardUser::add_user_to_board(
        &db.0,
        BoardUser {
            board_id: payload.board_id,
            user_id: payload.user_id,
            role: payload.role,
        },
    )
    .await
    .trace_err("Failed to add user to board")
    .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    Ok(StatusCode::CREATED)
}

#[derive(serde::Deserialize)]
pub struct UpdateUserRoleRequest {
    pub board_id: BoardId,
    pub user_id: UserId,
    pub role: BoardUserRole,
}

pub async fn update_user_role(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    Json(payload): Json<UpdateUserRoleRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let role = BoardUserRole::get_user_role_on_board(&db.0, payload.board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    ensure_minimum_role(
        role,
        BoardUserRole::Owner,
        "Only board owners can manage board members",
    )?;

    BoardUser::update_user_role_on_board(
        &db.0,
        BoardUser {
            board_id: payload.board_id,
            user_id: payload.user_id,
            role: payload.role,
        },
    )
    .await
    .trace_err("Failed to update user role")
    .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    Ok(StatusCode::OK)
}

#[derive(serde::Deserialize)]
pub struct RemoveUserRequest {
    pub board_id: BoardId,
    pub user_id: UserId,
}

pub async fn remove_user_from_board(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    Json(payload): Json<RemoveUserRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let role = BoardUserRole::get_user_role_on_board(&db.0, payload.board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    ensure_minimum_role(
        role,
        BoardUserRole::Owner,
        "Only board owners can manage board members",
    )?;

    BoardUser::remove_user_from_board(&db.0, payload.board_id, payload.user_id)
        .await
        .trace_err("Failed to remove user from board")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    Ok(StatusCode::OK)
}
