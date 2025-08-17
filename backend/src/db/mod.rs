pub async fn connect_env() -> sqlx::SqlitePool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    sqlx::SqlitePool::connect(&database_url).await.unwrap()
}
