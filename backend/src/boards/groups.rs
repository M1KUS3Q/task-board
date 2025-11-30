use sqlx::SqlitePool;

use crate::boards::board::{Board, BoardId};

pub type GroupId = i64;

pub struct BoardGroup {
    pub board_id: BoardId,
    pub name: String,
    pub position: String,
}

impl BoardGroup {
    /// Creates a new group associated with a specific board.
    /// After creating the group, it updates the board's `updated_at` timestamp.
    pub async fn create_group(db: &SqlitePool, group: BoardGroup) -> Result<GroupId, sqlx::Error> {
        let group_id = sqlx::query!(
            "INSERT INTO board_groups (board_id, name, position) VALUES (?, ?, ?) RETURNING id",
            group.board_id,
            group.name,
            group.position
        )
        .fetch_one(db)
        .await?
        .id;

        Board::update_board_timestamp(db, group.board_id).await?;
        Ok(group_id)
    }

    /// Retrieves a group by its ID.
    /// Returns `Ok(Some((GroupId, BoardId, name, position)))` if found, `Ok(None)` if not found.
    /// Returns an error if the database query fails.
    pub async fn get_group_by_id(
        db: &SqlitePool,
        group_id: GroupId,
    ) -> Result<Option<BoardGroup>, sqlx::Error> {
        let record = sqlx::query!(
            "SELECT id, board_id, name, position FROM board_groups WHERE id = ?",
            group_id
        )
        .fetch_optional(db)
        .await?;

        if let Some(rec) = record {
            Ok(Some(BoardGroup {
                board_id: rec.board_id,
                name: rec.name,
                position: rec.position,
            }))
        } else {
            Ok(None)
        }
    }

    /// Retrieves all groups associated with a specific board, ordered by their position.
    pub async fn get_all_groups_from_board(
        db: &SqlitePool,
        board_id: BoardId,
    ) -> Result<Vec<(GroupId, BoardGroup)>, sqlx::Error> {
        let groups = sqlx::query!(
            "SELECT id, name, position FROM board_groups WHERE board_id = ? ORDER BY position",
            board_id
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|record| {
            (
                record.id,
                BoardGroup {
                    board_id,
                    name: record.name,
                    position: record.position,
                },
            )
        })
        .collect();

        Ok(groups)
    }

    /// Updates the name of a group and updates the associated board's `updated_at` timestamp.
    /// If the group or board does not exist, returns an error.
    pub async fn update_group_name(
        db: &SqlitePool,
        group_id: GroupId,
        new_name: String,
    ) -> Result<(), sqlx::Error> {
        let board_id = sqlx::query!("SELECT board_id FROM board_groups WHERE id = ?", group_id)
            .fetch_one(db)
            .await?
            .board_id;

        sqlx::query!(
            "UPDATE board_groups SET name = ? WHERE id = ?",
            new_name,
            group_id
        )
        .execute(db)
        .await?;

        Board::update_board_timestamp(db, board_id).await?;
        Ok(())
    }

    /// Updates the position of a group and updates the associated board's `updated_at` timestamp.
    /// If the group or board does not exist, returns an error.
    pub async fn update_group_position(
        db: &SqlitePool,
        group_id: GroupId,
        new_position: String,
    ) -> Result<(), sqlx::Error> {
        let board_id = sqlx::query!("SELECT board_id FROM board_groups WHERE id = ?", group_id)
            .fetch_one(db)
            .await?
            .board_id;

        sqlx::query!(
            "UPDATE board_groups SET position = ? WHERE id = ?",
            new_position,
            group_id
        )
        .execute(db)
        .await?;

        Board::update_board_timestamp(db, board_id).await?;
        Ok(())
    }

    /// Deletes a group by its ID and updates the associated board's `updated_at` timestamp.
    /// If the group or board does not exist, returns an error.
    pub async fn delete_group(db: &SqlitePool, group_id: GroupId) -> Result<(), sqlx::Error> {
        let board_id = sqlx::query!("SELECT board_id FROM board_groups WHERE id = ?", group_id)
            .fetch_one(db)
            .await?
            .board_id;

        sqlx::query!("DELETE FROM board_groups WHERE id = ?", group_id)
            .execute(db)
            .await?;

        Board::update_board_timestamp(db, board_id).await?;
        Ok(())
    }
}
