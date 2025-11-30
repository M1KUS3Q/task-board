use sqlx::SqlitePool;

use crate::boards::{board::Board, groups::GroupId};

pub type CardId = i64;

pub struct Card {
    pub group_id: GroupId,
    pub title: String,
    pub content: Option<String>,
    pub position: String,
}

impl Card {
    /// Creates a new card in a group and updates the board's updated_at timestamp.
    pub async fn create_card(db: &SqlitePool, card: Card) -> Result<CardId, sqlx::Error> {
        let card_id = sqlx::query!(
            "INSERT INTO cards (group_id, title, content, position) VALUES (?, ?, ?, ?) RETURNING id",
            card.group_id,
            card.title,
            card.content,
            card.position
        )
        .fetch_one(db)
        .await?
        .id;

        let board_id = sqlx::query!(
            "SELECT board_id FROM board_groups WHERE id = ?",
            card.group_id
        )
        .fetch_one(db)
        .await?
        .board_id;

        Board::update_board_timestamp(db, board_id).await?;
        Ok(card_id)
    }

    /// Retrieves a card by its ID.
    pub async fn get_card_by_id(
        db: &SqlitePool,
        card_id: CardId,
    ) -> Result<Option<Card>, sqlx::Error> {
        let record = sqlx::query!(
            "SELECT group_id, title, content, position FROM cards WHERE id = ?",
            card_id
        )
        .fetch_optional(db)
        .await?;

        if let Some(rec) = record {
            Ok(Some(Card {
                group_id: rec.group_id,
                title: rec.title,
                content: rec.content,
                position: rec.position,
            }))
        } else {
            Ok(None)
        }
    }

    /// Retrieves all cards in a group, ordered by position.
    pub async fn get_all_cards_from_group(
        db: &SqlitePool,
        group_id: GroupId,
    ) -> Result<Vec<(CardId, Card)>, sqlx::Error> {
        let cards = sqlx::query!(
            "SELECT id, title, content, position FROM cards WHERE group_id = ? ORDER BY position",
            group_id
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|rec| {
            (
                rec.id,
                Card {
                    group_id,
                    title: rec.title,
                    content: rec.content,
                    position: rec.position,
                },
            )
        })
        .collect();

        Ok(cards)
    }

    /// Updates the title of a card and updates the board's updated_at timestamp.
    pub async fn update_card_title(
        db: &SqlitePool,
        card_id: CardId,
        new_title: String,
    ) -> Result<(), sqlx::Error> {
        let board_id = sqlx::query!(
            "SELECT bg.board_id FROM cards c JOIN board_groups bg ON c.group_id = bg.id WHERE c.id = ?",
            card_id
        )
        .fetch_one(db)
        .await?
        .board_id;

        sqlx::query!(
            "UPDATE cards SET title = ? WHERE id = ?",
            new_title,
            card_id
        )
        .execute(db)
        .await?;

        Board::update_board_timestamp(db, board_id).await?;
        Ok(())
    }

    /// Updates the content of a card and updates the board's updated_at timestamp.
    pub async fn update_card_content(
        db: &SqlitePool,
        card_id: CardId,
        new_content: Option<String>,
    ) -> Result<(), sqlx::Error> {
        let board_id = sqlx::query!(
            "SELECT bg.board_id FROM cards c JOIN board_groups bg ON c.group_id = bg.id WHERE c.id = ?",
            card_id
        )
        .fetch_one(db)
        .await?
        .board_id;

        sqlx::query!(
            "UPDATE cards SET content = ? WHERE id = ?",
            new_content,
            card_id
        )
        .execute(db)
        .await?;

        Board::update_board_timestamp(db, board_id).await?;
        Ok(())
    }

    /// Updates the position of a card and updates the board's updated_at timestamp.
    pub async fn update_card_position(
        db: &SqlitePool,
        card_id: CardId,
        new_position: String,
    ) -> Result<(), sqlx::Error> {
        let board_id = sqlx::query!(
            "SELECT bg.board_id FROM cards c JOIN board_groups bg ON c.group_id = bg.id WHERE c.id = ?",
            card_id
        )
        .fetch_one(db)
        .await?
        .board_id;

        sqlx::query!(
            "UPDATE cards SET position = ? WHERE id = ?",
            new_position,
            card_id
        )
        .execute(db)
        .await?;

        Board::update_board_timestamp(db, board_id).await?;
        Ok(())
    }

    /// Deletes a card by its ID and updates the board's updated_at timestamp.
    pub async fn delete_card(db: &SqlitePool, card_id: CardId) -> Result<(), sqlx::Error> {
        let board_id = sqlx::query!(
            "SELECT bg.board_id FROM cards c JOIN board_groups bg ON c.group_id = bg.id WHERE c.id = ?",
            card_id
        )
        .fetch_one(db)
        .await?
        .board_id;

        sqlx::query!("DELETE FROM cards WHERE id = ?", card_id)
            .execute(db)
            .await?;

        Board::update_board_timestamp(db, board_id).await?;
        Ok(())
    }
}
