use axum::{
    Router,
    extract::MatchedPath,
    http::{HeaderName, Method, Request, Response, StatusCode, header},
    routing::{get, post},
};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    classify::ServerErrorsFailureClass,
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::{Span, field};

use crate::{config::Config, games, realtime, state::AppState};

pub fn build_app(state: AppState, config: &Config) -> Router {
    let request_id_header = HeaderName::from_static("x-request-id");
    let cors = CorsLayer::new()
        .allow_origin([config.frontend_origin.parse().unwrap()])
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            request_id_header.clone(),
        ])
        .expose_headers([request_id_header.clone()]);

    let http_logging = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            let path = request
                .extensions()
                .get::<MatchedPath>()
                .map(MatchedPath::as_str)
                .unwrap_or("<unmatched>");
            let request_id = request
                .extensions()
                .get::<RequestId>()
                .and_then(|id| id.header_value().to_str().ok())
                .unwrap_or("<invalid>");

            tracing::info_span!(
                "http.request",
                method = %request.method(),
                path,
                request_id,
                status = field::Empty,
                latency_ms = field::Empty,
            )
        })
        .on_request(())
        .on_response(|response: &Response<_>, latency: Duration, span: &Span| {
            let status = response.status();
            span.record("status", status.as_u16());
            span.record("latency_ms", latency.as_secs_f64() * 1_000.0);

            match status {
                status if status.is_server_error() => {
                    tracing::error!("HTTP request completed")
                }
                status if status.is_client_error() => {
                    tracing::warn!("HTTP request completed")
                }
                StatusCode::SWITCHING_PROTOCOLS => {
                    tracing::info!("HTTP websocket upgrade completed")
                }
                _ => tracing::info!("HTTP request completed"),
            }
        })
        .on_failure(
            |failure: ServerErrorsFailureClass, latency: Duration, span: &Span| {
                span.record("latency_ms", latency.as_secs_f64() * 1_000.0);
                tracing::error!(?failure, "HTTP request failed before a response");
            },
        );

    Router::new()
        .route("/games/categories", get(games::api::list_categories))
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
            "/games/player/{player_code}/select-clue",
            post(games::api::select_clue_as_player),
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
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::new(
                    request_id_header.clone(),
                    MakeRequestUuid,
                ))
                .layer(PropagateRequestIdLayer::new(request_id_header))
                .layer(http_logging)
                .layer(cors),
        )
        .with_state(state)
}
