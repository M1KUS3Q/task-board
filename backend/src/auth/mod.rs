use axum::{
    Router,
    routing::{get, post},
};

pub type UserId = i32;

pub mod login;
pub mod me;
pub mod sample_protected_route;
pub mod signup;
pub mod utils;

pub fn router() -> Router {
    Router::new()
        .route("/login", post(login::login))
        .route("/signup", post(signup::signup))
        .route("/me", get(me::me))
        .route(
            "/sample-protected",
            get(sample_protected_route::protected_route),
        )
}
