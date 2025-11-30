use axum::{Json, http::StatusCode};
use axum_extra::{
    TypedHeader,
    headers::{self, Authorization},
};
use sqlx::SqlitePool;

use crate::{
    auth::utils,
    boards::{
        api::utils::{ensure_minimum_role, map_auth_error},
        board::BoardId,
        cards::{Card, CardId},
        groups::{BoardGroup, GroupId},
        role::BoardUserRole,
        utils::TraceError,
    },
};

async fn get_card_board_id(
    db: &SqlitePool,
    card_id: CardId,
) -> Result<Option<BoardId>, sqlx::Error> {
    let record = sqlx::query!(
        "SELECT bg.board_id FROM cards c JOIN board_groups bg ON c.group_id = bg.id WHERE c.id = ?",
        card_id
    )
    .fetch_optional(db)
    .await?;

    Ok(record.map(|rec| rec.board_id))
}

#[derive(serde::Deserialize)]
pub struct CardCreateRequest {
    pub group_id: GroupId,
    pub title: String,
    pub content: Option<String>,
    pub position: String,
}

pub async fn create_card(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    Json(payload): Json<CardCreateRequest>,
) -> Result<Json<CardId>, (StatusCode, String)> {
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
        "Only board owners or editors can modify cards",
    )?;

    let card_id = Card::create_card(
        &db.0,
        Card {
            group_id: payload.group_id,
            title: payload.title,
            content: payload.content,
            position: payload.position,
        },
    )
    .await
    .trace_err("Failed to create card")
    .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    Ok(Json(card_id))
}

#[derive(serde::Deserialize)]
pub struct CardDeleteRequest {
    pub card_id: CardId,
}

pub async fn delete_card(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    Json(payload): Json<CardDeleteRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let board_id = get_card_board_id(&db.0, payload.card_id)
        .await
        .trace_err("Failed to fetch card board")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?
        .ok_or((StatusCode::NOT_FOUND, "Card not found".to_string()))?;

    let role = BoardUserRole::get_user_role_on_board(&db.0, board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    ensure_minimum_role(
        role,
        BoardUserRole::Editor,
        "Only board owners or editors can modify cards",
    )?;

    Card::delete_card(&db.0, payload.card_id)
        .await
        .trace_err("Failed to delete card")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    Ok(StatusCode::OK)
}

#[derive(serde::Deserialize)]
pub struct CardUpdateRequest {
    pub card_id: CardId,
    pub title: Option<String>,
    pub content: Option<Option<String>>,
    pub position: Option<String>,
}

pub async fn update_card(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    Json(payload): Json<CardUpdateRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if payload.title.is_none() && payload.content.is_none() && payload.position.is_none() {
        return Err((StatusCode::BAD_REQUEST, "No fields to update".to_string()));
    }

    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let board_id = get_card_board_id(&db.0, payload.card_id)
        .await
        .trace_err("Failed to fetch card board")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?
        .ok_or((StatusCode::NOT_FOUND, "Card not found".to_string()))?;

    let role = BoardUserRole::get_user_role_on_board(&db.0, board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    ensure_minimum_role(
        role,
        BoardUserRole::Editor,
        "Only board owners or editors can modify cards",
    )?;

    if let Some(title) = payload.title {
        Card::update_card_title(&db.0, payload.card_id, title)
            .await
            .trace_err("Failed to update card title")
            .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;
    }

    if let Some(content) = payload.content {
        Card::update_card_content(&db.0, payload.card_id, content)
            .await
            .trace_err("Failed to update card content")
            .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;
    }

    if let Some(position) = payload.position {
        Card::update_card_position(&db.0, payload.card_id, position)
            .await
            .trace_err("Failed to update card position")
            .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;
    }

    Ok(StatusCode::OK)
}

#[derive(serde::Serialize)]
pub struct CardDetails {
    pub card_id: CardId,
    pub board_id: BoardId,
    pub group_id: GroupId,
    pub title: String,
    pub content: Option<String>,
    pub position: String,
}

pub async fn get_card_details(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    axum::extract::Path(card_id): axum::extract::Path<CardId>,
) -> Result<Json<CardDetails>, (StatusCode, String)> {
    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let card = Card::get_card_by_id(&db.0, card_id)
        .await
        .trace_err("Failed to fetch card")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?
        .ok_or((StatusCode::NOT_FOUND, "Card not found".to_string()))?;

    let board_id = get_card_board_id(&db.0, card_id)
        .await
        .trace_err("Failed to fetch card board")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?
        .ok_or((StatusCode::NOT_FOUND, "Card not found".to_string()))?;

    let role = BoardUserRole::get_user_role_on_board(&db.0, board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    ensure_minimum_role(role, BoardUserRole::Viewer, "No access to this board")?;

    Ok(Json(CardDetails {
        card_id,
        board_id,
        group_id: card.group_id,
        title: card.title,
        content: card.content,
        position: card.position,
    }))
}

#[derive(serde::Serialize)]
pub struct CardSummary {
    pub card_id: CardId,
    pub board_id: BoardId,
    pub group_id: GroupId,
    pub title: String,
    pub content: Option<String>,
    pub position: String,
}

pub async fn list_cards_for_group(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    axum::extract::Path(group_id): axum::extract::Path<GroupId>,
) -> Result<Json<Vec<CardSummary>>, (StatusCode, String)> {
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

    let cards = Card::get_all_cards_from_group(&db.0, group_id)
        .await
        .trace_err("Failed to fetch cards")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?
        .into_iter()
        .map(|(card_id, card)| CardSummary {
            card_id,
            board_id: group.board_id,
            group_id,
            title: card.title,
            content: card.content,
            position: card.position,
        })
        .collect();

    Ok(Json(cards))
}
