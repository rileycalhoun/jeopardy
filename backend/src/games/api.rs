use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
};

use crate::{
    error::AppResult,
    games::{
        auth,
        models::{
            AnswerRequest, CategoriesResponse, CreateGameResponse, FinishGameResponse,
            GameStateResponse, JoinAdminRequest, JoinGameRequest, LobbyResponse,
            PlayerAnswerRequest, PlayerSelectClueRequest, ResolveRequest, SelectClueRequest,
            StartGameRequest, WagerRequest,
        },
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

pub async fn list_categories(
    State(state): State<AppState>,
) -> AppResult<(StatusCode, Json<CategoriesResponse>)> {
    let categories = service::list_categories(&state)?;
    Ok((StatusCode::OK, Json(categories)))
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

pub async fn start_game(
    State(state): State<AppState>,
    Path(admin_code): Path<i32>,
    headers: HeaderMap,
    Json(request): Json<StartGameRequest>,
) -> AppResult<(StatusCode, Json<GameStateResponse>)> {
    let token = bearer(&headers);
    let game = service::start_game(&state, admin_code, token, request).await?;
    Ok((StatusCode::OK, Json(game)))
}

pub async fn get_admin_game_state(
    State(state): State<AppState>,
    Path(admin_code): Path<i32>,
    headers: HeaderMap,
) -> AppResult<(StatusCode, Json<GameStateResponse>)> {
    let game = service::game_state_by_admin_code(&state, admin_code, bearer(&headers)).await?;
    Ok((StatusCode::OK, Json(game)))
}

pub async fn get_player_game_state(
    State(state): State<AppState>,
    Path(player_code): Path<i32>,
) -> AppResult<(StatusCode, Json<GameStateResponse>)> {
    let game = service::game_state_by_player_code(&state, player_code).await?;
    Ok((StatusCode::OK, Json(game)))
}

pub async fn submit_player_answer(
    State(state): State<AppState>,
    Path(player_code): Path<i32>,
    headers: HeaderMap,
    Json(request): Json<PlayerAnswerRequest>,
) -> AppResult<(StatusCode, Json<GameStateResponse>)> {
    let game =
        service::submit_player_answer(&state, player_code, bearer(&headers), request).await?;
    Ok((StatusCode::OK, Json(game)))
}

pub async fn select_clue(
    State(state): State<AppState>,
    Path(admin_code): Path<i32>,
    headers: HeaderMap,
    Json(request): Json<SelectClueRequest>,
) -> AppResult<(StatusCode, Json<GameStateResponse>)> {
    let game = service::select_clue(&state, admin_code, bearer(&headers), request).await?;
    Ok((StatusCode::OK, Json(game)))
}

pub async fn select_clue_as_player(
    State(state): State<AppState>,
    Path(player_code): Path<i32>,
    headers: HeaderMap,
    Json(request): Json<PlayerSelectClueRequest>,
) -> AppResult<(StatusCode, Json<GameStateResponse>)> {
    let game =
        service::select_clue_as_player(&state, player_code, bearer(&headers), request).await?;
    Ok((StatusCode::OK, Json(game)))
}

pub async fn answer_clue(
    State(state): State<AppState>,
    Path(admin_code): Path<i32>,
    headers: HeaderMap,
    Json(request): Json<AnswerRequest>,
) -> AppResult<(StatusCode, Json<GameStateResponse>)> {
    let game = service::answer_clue(&state, admin_code, bearer(&headers), request).await?;
    Ok((StatusCode::OK, Json(game)))
}

pub async fn skip_clue(
    State(state): State<AppState>,
    Path(admin_code): Path<i32>,
    headers: HeaderMap,
) -> AppResult<(StatusCode, Json<GameStateResponse>)> {
    let game = service::skip_clue(&state, admin_code, bearer(&headers)).await?;
    Ok((StatusCode::OK, Json(game)))
}

pub async fn daily_double_wager(
    State(state): State<AppState>,
    Path(admin_code): Path<i32>,
    headers: HeaderMap,
    Json(request): Json<WagerRequest>,
) -> AppResult<(StatusCode, Json<GameStateResponse>)> {
    let game = service::daily_double_wager(&state, admin_code, bearer(&headers), request).await?;
    Ok((StatusCode::OK, Json(game)))
}

pub async fn daily_double_resolve(
    State(state): State<AppState>,
    Path(admin_code): Path<i32>,
    headers: HeaderMap,
    Json(request): Json<ResolveRequest>,
) -> AppResult<(StatusCode, Json<GameStateResponse>)> {
    let game = service::daily_double_resolve(&state, admin_code, bearer(&headers), request).await?;
    Ok((StatusCode::OK, Json(game)))
}

pub async fn final_wager(
    State(state): State<AppState>,
    Path(admin_code): Path<i32>,
    headers: HeaderMap,
    Json(request): Json<WagerRequest>,
) -> AppResult<(StatusCode, Json<GameStateResponse>)> {
    let game = service::final_wager(&state, admin_code, bearer(&headers), request).await?;
    Ok((StatusCode::OK, Json(game)))
}

pub async fn final_resolve(
    State(state): State<AppState>,
    Path(admin_code): Path<i32>,
    headers: HeaderMap,
    Json(request): Json<ResolveRequest>,
) -> AppResult<(StatusCode, Json<GameStateResponse>)> {
    let game = service::final_resolve(&state, admin_code, bearer(&headers), request).await?;
    Ok((StatusCode::OK, Json(game)))
}

pub async fn finish_game(
    State(state): State<AppState>,
    Path(admin_code): Path<i32>,
    headers: HeaderMap,
) -> AppResult<(StatusCode, Json<FinishGameResponse>)> {
    let response = service::finish_game(&state, admin_code, bearer(&headers)).await?;
    Ok((StatusCode::OK, Json(response)))
}

fn bearer(headers: &HeaderMap) -> Option<&str> {
    auth::bearer_token(
        headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok()),
    )
}
