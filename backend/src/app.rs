use axum::{
    Router,
    http::{Method, header},
    routing::{get, post},
};
use tower_http::cors::CorsLayer;

use crate::{config::Config, games, realtime, state::AppState};

pub fn build_app(state: AppState, config: &Config) -> Router {
    let cors = CorsLayer::new()
        .allow_origin([config.frontend_origin.parse().unwrap()])
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    Router::new()
        .route("/games/packs", get(games::api::list_question_packs))
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
        .route(
            "/games/admin/{admin_code}/start",
            post(games::api::start_game),
        )
        .route(
            "/games/admin/{admin_code}/state",
            get(games::api::get_admin_game_state),
        )
        .route(
            "/games/player/{player_code}/state",
            get(games::api::get_player_game_state),
        )
        .route(
            "/games/player/{player_code}/answer",
            post(games::api::submit_player_answer),
        )
        .route(
            "/games/admin/{admin_code}/select-clue",
            post(games::api::select_clue),
        )
        .route(
            "/games/admin/{admin_code}/answer",
            post(games::api::answer_clue),
        )
        .route(
            "/games/admin/{admin_code}/daily-double/wager",
            post(games::api::daily_double_wager),
        )
        .route(
            "/games/admin/{admin_code}/daily-double/resolve",
            post(games::api::daily_double_resolve),
        )
        .route(
            "/games/admin/{admin_code}/final/wager",
            post(games::api::final_wager),
        )
        .route(
            "/games/admin/{admin_code}/final/resolve",
            post(games::api::final_resolve),
        )
        .route(
            "/games/admin/{admin_code}/finish",
            post(games::api::finish_game),
        )
        .route(
            "/ws/games/admin/{admin_code}",
            get(realtime::ws::admin_socket),
        )
        .route(
            "/ws/games/player/{player_code}",
            get(realtime::ws::player_socket),
        )
        .layer(cors)
        .with_state(state)
}
