use sqlx::SqlitePool;

use crate::boards::board::Board;

pub type MetaId = i64;
type CardId = i64;

pub struct CardMetadata {
    pub item_id: CardId,
    pub key: String,
    pub value: Option<String>,
}

impl CardMetadata {
    /// Creates a metadata entry for a card and updates the board's updated_at timestamp.
    pub async fn create_metadata(
        db: &SqlitePool,
        metadata: CardMetadata,
    ) -> Result<MetaId, sqlx::Error> {
        let meta_id = sqlx::query!(
            "INSERT INTO card_metadata (item_id, key, value) VALUES (?, ?, ?) RETURNING id",
            metadata.item_id,
            metadata.key,
            metadata.value
        )
        .fetch_one(db)
        .await?
        .id;

        // derive board_id from card
        let board_id = sqlx::query!(
            "SELECT bg.board_id FROM cards c JOIN board_groups bg ON c.group_id = bg.id WHERE c.id = ?",
            metadata.item_id
        )
        .fetch_one(db)
        .await?
        .board_id;

        Board::update_board_timestamp(db, board_id).await?;
        Ok(meta_id)
    }

    /// Retrieves a metadata entry by its ID.
    pub async fn get_metadata_by_id(
        db: &SqlitePool,
        meta_id: MetaId,
    ) -> Result<Option<CardMetadata>, sqlx::Error> {
        let record = sqlx::query!(
            "SELECT item_id, key, value FROM card_metadata WHERE id = ?",
            meta_id
        )
        .fetch_optional(db)
        .await?;

        if let Some(rec) = record {
            Ok(Some(CardMetadata {
                item_id: rec.item_id,
                key: rec.key,
                value: rec.value,
            }))
        } else {
            Ok(None)
        }
    }

    /// Retrieves all metadata entries for a specific card.
    pub async fn get_all_metadata_for_card(
        db: &SqlitePool,
        item_id: CardId,
    ) -> Result<Vec<(MetaId, CardMetadata)>, sqlx::Error> {
        let metas = sqlx::query!(
            "SELECT id, key, value FROM card_metadata WHERE item_id = ?",
            item_id
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|rec| {
            (
                rec.id,
                CardMetadata {
                    item_id,
                    key: rec.key,
                    value: rec.value,
                },
            )
        })
        .collect();

        Ok(metas)
    }

    /// Updates the value of a metadata entry and updates the board's updated_at timestamp.
    pub async fn update_metadata_value(
        db: &SqlitePool,
        meta_id: MetaId,
        new_value: Option<String>,
    ) -> Result<(), sqlx::Error> {
        // derive board_id through joins
        let board_id = sqlx::query!(
            "SELECT bg.board_id FROM card_metadata cm JOIN cards c ON cm.item_id = c.id JOIN board_groups bg ON c.group_id = bg.id WHERE cm.id = ?",
            meta_id
        )
        .fetch_one(db)
        .await?
        .board_id;

        sqlx::query!(
            "UPDATE card_metadata SET value = ? WHERE id = ?",
            new_value,
            meta_id
        )
        .execute(db)
        .await?;

        Board::update_board_timestamp(db, board_id).await?;
        Ok(())
    }

    /// Deletes a metadata entry and updates the board's updated_at timestamp.
    pub async fn delete_metadata(db: &SqlitePool, meta_id: MetaId) -> Result<(), sqlx::Error> {
        // derive board_id through joins
        let board_id = sqlx::query!(
            "SELECT bg.board_id FROM card_metadata cm JOIN cards c ON cm.item_id = c.id JOIN board_groups bg ON c.group_id = bg.id WHERE cm.id = ?",
            meta_id
        )
        .fetch_one(db)
        .await?
        .board_id;

        sqlx::query!("DELETE FROM card_metadata WHERE id = ?", meta_id)
            .execute(db)
            .await?;

        Board::update_board_timestamp(db, board_id).await?;
        Ok(())
    }
}
