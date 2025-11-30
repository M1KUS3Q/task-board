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
        card_metadata::{CardMetadata, MetaId},
        cards::CardId,
        role::BoardUserRole,
        utils::TraceError,
    },
};

async fn board_id_from_card(
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

async fn board_id_from_metadata(
    db: &SqlitePool,
    meta_id: MetaId,
) -> Result<Option<BoardId>, sqlx::Error> {
    let record = sqlx::query!(
        "SELECT bg.board_id FROM card_metadata cm JOIN cards c ON cm.item_id = c.id JOIN board_groups bg ON c.group_id = bg.id WHERE cm.id = ?",
        meta_id
    )
    .fetch_optional(db)
    .await?;

    Ok(record.map(|rec| rec.board_id))
}

#[derive(serde::Deserialize)]
pub struct MetadataCreateRequest {
    pub card_id: CardId,
    pub key: String,
    pub value: Option<String>,
}

pub async fn create_metadata(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    Json(payload): Json<MetadataCreateRequest>,
) -> Result<Json<MetaId>, (StatusCode, String)> {
    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let board_id = board_id_from_card(&db.0, payload.card_id)
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
        "Only board owners or editors can modify card metadata",
    )?;

    let meta_id = CardMetadata::create_metadata(
        &db.0,
        CardMetadata {
            item_id: payload.card_id,
            key: payload.key,
            value: payload.value,
        },
    )
    .await
    .trace_err("Failed to create metadata")
    .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    Ok(Json(meta_id))
}

#[derive(serde::Deserialize)]
pub struct MetadataDeleteRequest {
    pub meta_id: MetaId,
}

pub async fn delete_metadata(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    Json(payload): Json<MetadataDeleteRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let board_id = board_id_from_metadata(&db.0, payload.meta_id)
        .await
        .trace_err("Failed to fetch metadata board")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?
        .ok_or((
            StatusCode::NOT_FOUND,
            "Metadata entry not found".to_string(),
        ))?;

    let role = BoardUserRole::get_user_role_on_board(&db.0, board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    ensure_minimum_role(
        role,
        BoardUserRole::Editor,
        "Only board owners or editors can modify card metadata",
    )?;

    CardMetadata::delete_metadata(&db.0, payload.meta_id)
        .await
        .trace_err("Failed to delete metadata")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    Ok(StatusCode::OK)
}

#[derive(serde::Deserialize)]
pub struct MetadataUpdateRequest {
    pub meta_id: MetaId,
    pub value: Option<Option<String>>,
}

pub async fn update_metadata_value(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    Json(payload): Json<MetadataUpdateRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if payload.value.is_none() {
        return Err((StatusCode::BAD_REQUEST, "No fields to update".to_string()));
    }

    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let board_id = board_id_from_metadata(&db.0, payload.meta_id)
        .await
        .trace_err("Failed to fetch metadata board")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?
        .ok_or((
            StatusCode::NOT_FOUND,
            "Metadata entry not found".to_string(),
        ))?;

    let role = BoardUserRole::get_user_role_on_board(&db.0, board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    ensure_minimum_role(
        role,
        BoardUserRole::Editor,
        "Only board owners or editors can modify card metadata",
    )?;

    if let Some(value) = payload.value {
        CardMetadata::update_metadata_value(&db.0, payload.meta_id, value)
            .await
            .trace_err("Failed to update metadata value")
            .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;
    }

    Ok(StatusCode::OK)
}

#[derive(serde::Serialize)]
pub struct MetadataDetails {
    pub meta_id: MetaId,
    pub board_id: BoardId,
    pub card_id: CardId,
    pub key: String,
    pub value: Option<String>,
}

pub async fn get_metadata_details(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    axum::extract::Path(meta_id): axum::extract::Path<MetaId>,
) -> Result<Json<MetadataDetails>, (StatusCode, String)> {
    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let metadata = CardMetadata::get_metadata_by_id(&db.0, meta_id)
        .await
        .trace_err("Failed to fetch metadata entry")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?
        .ok_or((
            StatusCode::NOT_FOUND,
            "Metadata entry not found".to_string(),
        ))?;

    let board_id = board_id_from_metadata(&db.0, meta_id)
        .await
        .trace_err("Failed to fetch metadata board")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?
        .ok_or((
            StatusCode::NOT_FOUND,
            "Metadata entry not found".to_string(),
        ))?;

    let role = BoardUserRole::get_user_role_on_board(&db.0, board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    ensure_minimum_role(role, BoardUserRole::Viewer, "No access to this board")?;

    Ok(Json(MetadataDetails {
        meta_id,
        board_id,
        card_id: metadata.item_id,
        key: metadata.key,
        value: metadata.value,
    }))
}

#[derive(serde::Serialize)]
pub struct MetadataSummary {
    pub meta_id: MetaId,
    pub board_id: BoardId,
    pub card_id: CardId,
    pub key: String,
    pub value: Option<String>,
}

pub async fn list_metadata_for_card(
    db: axum::extract::Extension<SqlitePool>,
    TypedHeader(auth): TypedHeader<Authorization<headers::authorization::Bearer>>,
    axum::extract::Path(card_id): axum::extract::Path<CardId>,
) -> Result<Json<Vec<MetadataSummary>>, (StatusCode, String)> {
    let user_id = utils::extract_bearer(auth).map_err(map_auth_error)?;

    let board_id = board_id_from_card(&db.0, card_id)
        .await
        .trace_err("Failed to fetch card board")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?
        .ok_or((StatusCode::NOT_FOUND, "Card not found".to_string()))?;

    let role = BoardUserRole::get_user_role_on_board(&db.0, board_id, user_id)
        .await
        .trace_err("Couldn't get user's board role")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?;

    ensure_minimum_role(role, BoardUserRole::Viewer, "No access to this board")?;

    let metadata = CardMetadata::get_all_metadata_for_card(&db.0, card_id)
        .await
        .trace_err("Failed to fetch metadata entries")
        .map_err(|msg| (StatusCode::INTERNAL_SERVER_ERROR, msg))?
        .into_iter()
        .map(|(meta_id, entry)| MetadataSummary {
            meta_id,
            board_id,
            card_id,
            key: entry.key,
            value: entry.value,
        })
        .collect();

    Ok(Json(metadata))
}
