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
        groups::{BoardGroup, GroupId},
        role::BoardUserRole,
        utils::TraceError,
    },
};

#[derive(serde::Deserialize)]
pub struct GroupCreateRequest {
    pub board_id: BoardId,
    pub name: String,
    pub position: String,
}

pub async fn create_group(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    Json(payload): Json<GroupCreateRequest>,
) -> Result<Json<GroupId>, (StatusCode, String)> {
    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let role = BoardUserRole::get_user_role_on_board(&db.0, payload.board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    ensure_minimum_role(
        role,
        BoardUserRole::Editor,
        "Only board owners or editors can modify groups",
    )?;

    let group_id = BoardGroup::create_group(
        &db.0,
        BoardGroup {
            board_id: payload.board_id,
            name: payload.name,
            position: payload.position,
        },
    )
    .await
    .trace_err("Failed to create group")
    .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    Ok(Json(group_id))
}

#[derive(serde::Deserialize)]
pub struct GroupDeleteRequest {
    pub group_id: GroupId,
}

pub async fn delete_group(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    Json(payload): Json<GroupDeleteRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let group = BoardGroup::get_group_by_id(&db.0, payload.group_id)
        .await
        .trace_err("Failed to fetch group")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?
        .ok_or((StatusCode::NOT_FOUND, "Group not found".to_string()))?;

    let role = BoardUserRole::get_user_role_on_board(&db.0, group.board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    ensure_minimum_role(
        role,
        BoardUserRole::Editor,
        "Only board owners or editors can modify groups",
    )?;

    BoardGroup::delete_group(&db.0, payload.group_id)
        .await
        .trace_err("Failed to delete group")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    Ok(StatusCode::OK)
}

#[derive(serde::Deserialize)]
pub struct GroupUpdateRequest {
    pub group_id: GroupId,
    pub name: Option<String>,
    pub position: Option<String>,
}

pub async fn update_group(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    Json(payload): Json<GroupUpdateRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if payload.name.is_none() && payload.position.is_none() {
        return Err((StatusCode::BAD_REQUEST, "No fields to update".to_string()));
    }

    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let group = BoardGroup::get_group_by_id(&db.0, payload.group_id)
        .await
        .trace_err("Failed to fetch group")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?
        .ok_or((StatusCode::NOT_FOUND, "Group not found".to_string()))?;

    let role = BoardUserRole::get_user_role_on_board(&db.0, group.board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    ensure_minimum_role(
        role,
        BoardUserRole::Editor,
        "Only board owners or editors can modify groups",
    )?;

    if let Some(name) = payload.name {
        BoardGroup::update_group_name(&db.0, payload.group_id, name)
            .await
            .trace_err("Failed to update group name")
            .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;
    }

    if let Some(position) = payload.position {
        BoardGroup::update_group_position(&db.0, payload.group_id, position)
            .await
            .trace_err("Failed to update group position")
            .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;
    }

    Ok(StatusCode::OK)
}

#[derive(serde::Serialize)]
pub struct GroupDetails {
    pub group_id: GroupId,
    pub board_id: BoardId,
    pub name: String,
    pub position: String,
}

pub async fn get_group_details(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    axum::extract::Path(group_id): axum::extract::Path<GroupId>,
) -> Result<Json<GroupDetails>, (StatusCode, String)> {
    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let group = BoardGroup::get_group_by_id(&db.0, group_id)
        .await
        .trace_err("Failed to fetch group")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?
        .ok_or((StatusCode::NOT_FOUND, "Group not found".to_string()))?;

    let role = BoardUserRole::get_user_role_on_board(&db.0, group.board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    ensure_minimum_role(role, BoardUserRole::Viewer, "No access to this board")?;

    Ok(Json(GroupDetails {
        group_id,
        board_id: group.board_id,
        name: group.name,
        position: group.position,
    }))
}

#[derive(serde::Serialize)]
pub struct GroupSummary {
    pub group_id: GroupId,
    pub board_id: BoardId,
    pub name: String,
    pub position: String,
}

pub async fn list_board_groups(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    axum::extract::Path(board_id): axum::extract::Path<BoardId>,
) -> Result<Json<Vec<GroupSummary>>, (StatusCode, String)> {
    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let role = BoardUserRole::get_user_role_on_board(&db.0, board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    ensure_minimum_role(role, BoardUserRole::Viewer, "No access to this board")?;

    let groups = BoardGroup::get_all_groups_from_board(&db.0, board_id)
        .await
        .trace_err("Failed to fetch groups")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?
        .into_iter()
        .map(|(group_id, group)| GroupSummary {
            group_id,
            board_id,
            name: group.name,
            position: group.position,
        })
        .collect();

    Ok(Json(groups))
}
