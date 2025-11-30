use sqlx::SqlitePool;

///  A utility struct to verify the existence of database records.
/// For now this is not strictly necessary, as the database constraints should handle this,
/// but it can be useful for providing more user-friendly error messages.
pub struct Verificator<'a>(&'a SqlitePool);
impl<'a> Verificator<'a> {
    pub fn new(db: &'a SqlitePool) -> Self {
        Self(db)
    }

    pub async fn user_exists(&self, user_id: i32) -> Result<bool, sqlx::Error> {
        let record = sqlx::query!("SELECT id FROM users WHERE id = ?", user_id)
            .fetch_optional(self.0)
            .await?;
        Ok(record.is_some())
    }

    pub async fn board_exists(&self, board_id: i64) -> Result<bool, sqlx::Error> {
        let record = sqlx::query!("SELECT id FROM boards WHERE id = ?", board_id)
            .fetch_optional(self.0)
            .await?;
        Ok(record.is_some())
    }
}
