use sqlx::SqlitePool;

use crate::boards::{board::BoardId, role::BoardUserRole};

pub type UserId = i64;

pub struct BoardUser {
    pub board_id: BoardId,
    pub user_id: UserId,
    pub role: BoardUserRole,
}

impl BoardUser {
    pub async fn get_users_on_board(
        db: &SqlitePool,
        board_id: BoardId,
    ) -> Result<Vec<BoardUser>, sqlx::Error> {
        let users = sqlx::query_as!(
            BoardUser,
            "SELECT board_id, user_id, role FROM board_users WHERE board_id = ?",
            board_id
        )
        .fetch_all(db)
        .await?;

        Ok(users)
    }

    pub async fn add_user_to_board(db: &SqlitePool, user: BoardUser) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO board_users (board_id, user_id, role) VALUES (?, ?, ?)",
            user.board_id,
            user.user_id,
            user.role
        )
        .execute(db)
        .await?;
        Ok(())
    }

    pub async fn update_user_role_on_board(
        db: &SqlitePool,
        user: BoardUser,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE board_users SET role = ? WHERE board_id = ? AND user_id = ?",
            user.role,
            user.board_id,
            user.user_id
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn remove_user_from_board(
        db: &SqlitePool,
        board_id: BoardId,
        user_id: UserId,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM board_users WHERE board_id = ? AND user_id = ?",
            board_id,
            user_id
        )
        .execute(db)
        .await?;

        Ok(())
    }
}
