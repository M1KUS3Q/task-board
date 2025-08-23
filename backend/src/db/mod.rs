use sqlx::migrate::MigrateDatabase;

pub async fn connect_env() -> sqlx::SqlitePool {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // if exists, connect and return the pool
    if sqlx::Sqlite::database_exists(&db_url)
        .await
        .unwrap_or(false)
    {
        return sqlx::SqlitePool::connect(&db_url).await.unwrap();
    }

    // else, create the database, run migrations, and return the pool
    sqlx::Sqlite::create_database(&db_url).await.unwrap();
    let pool = sqlx::SqlitePool::connect(&db_url).await.unwrap();
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    pool
}
