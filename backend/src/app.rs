use axum::{
    Router,
    http::{Method, header},
    routing::{get, post},
};
use tower_http::cors::CorsLayer;

use crate::{config::Config, games, state::AppState};

pub fn build_app(state: AppState, config: &Config) -> Router {
    let cors = CorsLayer::new()
        .allow_origin([config.frontend_origin.parse().unwrap()])
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    Router::new()
        .route("/games/new", post(games::api::create_new_game))
        .route("/games/join", post(games::api::join_game_as_player))
        .route("/games/join/player", post(games::api::join_game_as_player))
        .route("/games/join/admin", post(games::api::join_game_as_admin))
        .route(
            "/games/player/{player_code}",
            get(games::api::get_lobby_by_player_code),
        )
        .route(
            "/games/admin/{admin_code}",
            get(games::api::get_lobby_by_admin_code),
        )
        .layer(cors)
        .with_state(state)
}
