use sqlx::SqlitePool;

use crate::boards::role::BoardUserRole;

const CREATOR_ROLE: BoardUserRole = BoardUserRole::Owner;
pub type BoardId = i64;

pub struct Board {
    pub id: BoardId,
    pub name: String,
    pub description: Option<String>,
}

impl Board {
    /// Updates the `updated_at` timestamp of the board to the current time.
    pub async fn update_board_timestamp(
        db: &SqlitePool,
        board_id: BoardId,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE boards SET updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            board_id
        )
        .execute(db)
        .await?;
        Ok(())
    }

    /// Creates a new board and associates it with the given user as the owner, as defined by `CREATOR_ROLE`.
    pub async fn create_board(
        db: &SqlitePool,
        user_id: i64,
        name: String,
    ) -> Result<BoardId, sqlx::Error> {
        // This ensures that if any part of the process fails, no partial data is committed
        // to the database, as opposed to doing it one by one.
        let mut tx = db.begin().await?;

        // Insert the new board
        let board_id = sqlx::query!("INSERT INTO boards (name) VALUES (?) RETURNING id", name)
            .fetch_one(&mut *tx)
            .await?
            .id;

        // Associate the board with the user
        let _ = sqlx::query!(
            "INSERT INTO board_users (board_id, user_id, role) VALUES (?, ?, ?)",
            board_id,
            user_id,
            CREATOR_ROLE
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(board_id)
    }

    /// Updates the name of the board with the given `board_id`.
    pub async fn update_board_name(
        db: &SqlitePool,
        board_id: BoardId,
        name: String,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!("UPDATE boards SET name = ? WHERE id = ?", name, board_id)
            .execute(db)
            .await?;

        Self::update_board_timestamp(db, board_id).await?;
        Ok(())
    }

    /// Updates the description of the board with the given `board_id`.
    pub async fn update_board_description(
        db: &SqlitePool,
        board_id: BoardId,
        description: String,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE boards SET description = ? WHERE id = ?",
            description,
            board_id
        )
        .execute(db)
        .await?;

        Self::update_board_timestamp(db, board_id).await?;
        Ok(())
    }

    /// Deletes the board with the given `board_id`.
    /// Also removes all associations with users, groups and cards, due to foreign key constraints with `ON DELETE CASCADE`.
    pub async fn delete_board(db: &SqlitePool, board_id: BoardId) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM boards WHERE id = ?", board_id)
            .execute(db)
            .await?;
        Ok(())
    }

    /// Retrieves a board by its ID.
    pub async fn get_board(
        db: &SqlitePool,
        board_id: BoardId,
    ) -> Result<Option<Board>, sqlx::Error> {
        let board = sqlx::query_as!(
            Board,
            "SELECT id, name, description FROM boards WHERE id = ?",
            board_id
        )
        .fetch_optional(db)
        .await?;

        Ok(board)
    }
}
