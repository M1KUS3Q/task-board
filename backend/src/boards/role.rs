use serde::{Deserialize, Serialize};

use crate::boards::board::BoardId;

#[derive(
    Clone, Copy, Serialize, Deserialize, Debug, sqlx::Type, PartialEq, Eq, PartialOrd, Ord,
)]
#[sqlx(type_name = "role", rename_all = "lowercase")]
pub enum BoardUserRole {
    Owner = 2,
    Editor = 1,
    Viewer = 0,
}

impl BoardUserRole {
    pub async fn get_user_role_on_board(
        db: &sqlx::SqlitePool,
        board_id: BoardId,
        user_id: i64,
    ) -> Result<Option<BoardUserRole>, sqlx::Error> {
        let role_str = sqlx::query_scalar!(
            "SELECT role FROM board_users WHERE board_id = ? AND user_id = ?",
            board_id,
            user_id
        )
        .fetch_optional(db)
        .await?;

        Ok(role_str.map(BoardUserRole::from))
    }
    pub fn is_at_least(&self, required: BoardUserRole) -> bool {
        *self >= required
    }
}

impl From<String> for BoardUserRole {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "owner" => BoardUserRole::Owner,
            "editor" => BoardUserRole::Editor,
            "viewer" => BoardUserRole::Viewer,
            other => {
                // TODO: debate if this should panic! instead
                tracing::warn!("Unknown role string present in db: {}", other);
                BoardUserRole::Viewer // Default to Viewer if unknown
            }
        }
    }
}
