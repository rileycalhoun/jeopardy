use crate::{
    error::AppError,
    games::{
        codes,
        models::{GameData, JoinAdminRequest, JoinGameRequest, LobbyResponse},
        repository::{self, find_game_by_admin_code, find_game_by_player_code},
    },
    players,
    state::AppState,
};

pub(crate) async fn create_new_game(state: &AppState) -> Result<GameData, AppError> {
    for _ in 0..10 {
        // Generate new game_id
        let game_id: String = codes::generate_game_id();

        // Generate new admin_code
        let admin_code: i32 = codes::generate_admin_code();

        // Generate new player_code
        let player_code: i32 = codes::generate_player_code();

        match repository::insert_game(&state.pool, game_id, admin_code, player_code).await {
            Ok(Some(game)) => return Ok(game),
            Ok(None) => continue,
            Err(err) => return Err(AppError::Database(err)),
        }
    }

    Err(AppError::GameCreationExhausted)
}

pub(crate) async fn join_game(
    state: &AppState,
    request: JoinGameRequest,
) -> Result<LobbyResponse, AppError> {
    let game = match find_game_by_player_code(&state.pool, request.player_code).await {
        Ok(Some(game)) => game,
        Ok(None) => return Err(AppError::GameNotFound),
        Err(err) => return Err(AppError::Database(err)),
    };

    match players::repository::insert_player(&state.pool, game.id, request.display_name).await {
        Ok(_) => {}
        Err(err) => {
            if let Some(db_err) = err.as_database_error() {
                if db_err.is_unique_violation() {
                    return Err(AppError::DuplicatePlayerName);
                }
            }

            return Err(AppError::Database(err));
        }
    }

    let players = match players::repository::list_players_for_game(&state.pool, game.id).await {
        Ok(players) => players,
        Err(err) => return Err(AppError::Database(err)),
    };

    Ok(LobbyResponse { players })
}

pub(crate) async fn get_lobby_by_player_code(
    state: &AppState,
    player_code: i32,
) -> Result<LobbyResponse, AppError> {
    let game = match repository::find_game_by_player_code(&state.pool, player_code).await {
        Ok(Some(game)) => game,
        Ok(None) => return Err(AppError::GameNotFound),
        Err(err) => return Err(AppError::Database(err)),
    };

    let players = match players::repository::list_players_for_game(&state.pool, game.id).await {
        Ok(players) => players,
        Err(err) => return Err(AppError::Database(err)),
    };

    Ok(LobbyResponse { players })
}

pub(crate) async fn join_game_as_admin(
    state: &AppState,
    request: JoinAdminRequest,
) -> Result<LobbyResponse, AppError> {
    let game = match find_game_by_admin_code(&state.pool, request.admin_code).await {
        Ok(Some(game)) => game,
        Ok(None) => return Err(AppError::GameNotFound),
        Err(err) => return Err(AppError::Database(err)),
    };

    let players = match players::repository::list_players_for_game(&state.pool, game.id).await {
        Ok(players) => players,
        Err(err) => return Err(AppError::Database(err)),
    };

    Ok(LobbyResponse { players })
}

pub(crate) async fn get_lobby_by_admin_code(
    state: &AppState,
    admin_code: i32,
) -> Result<LobbyResponse, AppError> {
    let game = match repository::find_game_by_admin_code(&state.pool, admin_code).await {
        Ok(Some(game)) => game,
        Ok(None) => return Err(AppError::GameNotFound),
        Err(err) => return Err(AppError::Database(err)),
    };

    let players = match players::repository::list_players_for_game(&state.pool, game.id).await {
        Ok(players) => players,
        Err(err) => return Err(AppError::Database(err)),
    };

    Ok(LobbyResponse { players })
}
