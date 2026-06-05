use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    error::AppResult,
    games::{
        models::{CreateGameResponse, JoinAdminRequest, JoinGameRequest, LobbyResponse},
        service,
    },
    state::AppState,
};

pub async fn create_new_game(
    State(state): State<AppState>,
) -> AppResult<(StatusCode, Json<CreateGameResponse>)> {
    let game = service::create_new_game(&state).await?;
    Ok((
        StatusCode::OK,
        Json(CreateGameResponse {
            admin_code: game.admin_code,
            player_code: game.player_code,
        }),
    ))
}

pub async fn join_game_as_player(
    State(state): State<AppState>,
    Json(request): Json<JoinGameRequest>,
) -> AppResult<(StatusCode, Json<LobbyResponse>)> {
    let lobby = service::join_game(&state, request).await?;
    Ok((StatusCode::OK, Json(lobby)))
}

pub async fn join_game_as_admin(
    State(state): State<AppState>,
    Json(request): Json<JoinAdminRequest>,
) -> AppResult<(StatusCode, Json<LobbyResponse>)> {
    let lobby = service::join_game_as_admin(&state, request).await?;
    Ok((StatusCode::OK, Json(lobby)))
}

pub async fn get_lobby_by_player_code(
    State(state): State<AppState>,
    Path(player_code): Path<i32>,
) -> AppResult<(StatusCode, Json<LobbyResponse>)> {
    let lobby = service::get_lobby_by_player_code(&state, player_code).await?;
    Ok((StatusCode::OK, Json(lobby)))
}

pub async fn get_lobby_by_admin_code(
    State(state): State<AppState>,
    Path(admin_code): Path<i32>,
) -> AppResult<(StatusCode, Json<LobbyResponse>)> {
    let lobby = service::get_lobby_by_admin_code(&state, admin_code).await?;
    Ok((StatusCode::OK, Json(lobby)))
}
