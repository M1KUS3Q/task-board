use axum::{
    Router,
    routing::{get, post},
};

pub mod board;
pub mod card_metadata;
pub mod cards;
pub mod groups;
pub mod users;
mod utils;

pub fn router() -> Router {
    Router::new()
        .route("/create_board", post(board::create_board))
        .route("/delete_board", post(board::delete_board))
        .route("/update_board", post(board::update_board_details))
        .route("/get_board/{board_id}", get(board::get_board_details))
        .route("/groups/create_group", post(groups::create_group))
        .route("/groups/delete_group", post(groups::delete_group))
        .route("/groups/update_group", post(groups::update_group))
        .route(
            "/groups/get_group/{group_id}",
            get(groups::get_group_details),
        )
        .route(
            "/groups/by_board/{board_id}",
            get(groups::list_board_groups),
        )
        .route("/cards/create_card", post(cards::create_card))
        .route("/cards/delete_card", post(cards::delete_card))
        .route("/cards/update_card", post(cards::update_card))
        .route("/cards/get_card/{card_id}", get(cards::get_card_details))
        .route(
            "/cards/by_group/{group_id}",
            get(cards::list_cards_for_group),
        )
        .route("/users/list/{board_id}", get(users::list_users_on_board))
        .route("/users/add_user", post(users::add_user_to_board))
        .route("/users/update_role", post(users::update_user_role))
        .route("/users/remove_user", post(users::remove_user_from_board))
        .route(
            "/card_metadata/create",
            post(card_metadata::create_metadata),
        )
        .route(
            "/card_metadata/update",
            post(card_metadata::update_metadata_value),
        )
        .route(
            "/card_metadata/delete",
            post(card_metadata::delete_metadata),
        )
        .route(
            "/card_metadata/get/{meta_id}",
            get(card_metadata::get_metadata_details),
        )
        .route(
            "/card_metadata/by_card/{card_id}",
            get(card_metadata::list_metadata_for_card),
        )
}
