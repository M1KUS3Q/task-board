pub mod auth;
pub mod db;

use axum::{Json, Router, http::StatusCode, routing};
use serde::{Deserialize, Serialize};

use crate::auth::signup::create_user;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    dotenvy::dotenv().ok();

    let pool = db::connect_env().await;
    let count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await?;
    println!("Number of users: {count}");

    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", routing::get(root))
        // `POST /users` goes to `create_user`
        .route("/users", routing::post(create_user));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}
