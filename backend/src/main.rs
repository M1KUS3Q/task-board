pub mod auth;
pub mod db;

use axum::extract::Extension;
use axum::{Router, routing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let pool = db::connect_env().await;

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();

    tracing::info!("Database connection established");

    let app = Router::new()
        .route("/", routing::get(root))
        .route("/api/health", routing::get(|| async { "OK" }))
        .nest("/api/auth", auth::router())
        .layer(Extension(pool));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}
