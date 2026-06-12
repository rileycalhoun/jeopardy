use crate::{
    content::service::build_scenario,
    domain::jeopardy::{GameAction, GamePhase, JeopardyGame},
    error::AppError,
    games::{
        auth, codes,
        models::{
            AnswerRequest, FinishGameResponse, GameData, GameStateResponse, JoinAdminRequest,
            JoinGameRequest, LobbyResponse, PlayerAnswerRequest, QuestionPacksResponse,
            ResolveRequest, SelectClueRequest, StartGameRequest, WagerRequest,
        },
        repository::{self, find_game_by_admin_code, find_game_by_player_code},
        state::build_game_view,
    },
    players,
    realtime::{self, UpdateKind},
    sessions::{runtime::RuntimeSession, store::SessionError},
    state::AppState,
};
use tracing::info;

pub(crate) async fn create_new_game(state: &AppState) -> Result<GameData, AppError> {
    for _ in 0..10 {
        // Generate new game_id
        let game_id: String = codes::generate_game_id();

        // Generate new admin_code
        let admin_code: i32 = codes::generate_admin_code();

        // Generate new player_code
        let player_code: i32 = codes::generate_player_code();

        match repository::insert_game(&state.pool, game_id, admin_code, player_code).await {
            Ok(Some(game)) => {
                info!("game created");
                return Ok(game);
            }
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

    let joined_player = match players::repository::insert_player(
        &state.pool,
        game.id,
        request.display_name,
    )
    .await
    {
        Ok(player) => player,
        Err(err) => {
            if let Some(db_err) = err.as_database_error() {
                if db_err.is_unique_violation() {
                    return Err(AppError::DuplicatePlayerName);
                }
            }

            return Err(AppError::Database(err));
        }
    };

    let players = match players::repository::list_players_for_game(&state.pool, game.id).await {
        Ok(players) => players,
        Err(err) => {
            return Err(AppError::Database(err));
        }
    };

    realtime::notify(state, game.id, UpdateKind::Lobby).await;
    info!(
        game_id = game.id,
        player_id = joined_player.id,
        player_count = players.len(),
        "player joined game"
    );

    Ok(LobbyResponse {
        players,
        admin_token: None,
        current_player_id: Some(joined_player.id),
    })
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

    Ok(LobbyResponse {
        players,
        admin_token: None,
        current_player_id: None,
    })
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

    let token = auth::generate_admin_token();
    let token_hash = auth::hash_admin_token(&token);
    repository::insert_admin_token(&state.pool, game.id, token_hash, "admin".to_owned())
        .await
        .map_err(AppError::Database)?;
    info!(game_id = game.id, "admin joined game");

    Ok(LobbyResponse {
        players,
        admin_token: Some(token),
        current_player_id: None,
    })
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

    Ok(LobbyResponse {
        players,
        admin_token: None,
        current_player_id: None,
    })
}

pub(crate) fn list_question_packs(state: &AppState) -> Result<QuestionPacksResponse, AppError> {
    let packs = state
        .question_packs
        .list()
        .map_err(AppError::QuestionPack)?;
    Ok(QuestionPacksResponse { packs })
}

pub(crate) async fn start_game(
    state: &AppState,
    admin_code: i32,
    token: Option<&str>,
    request: StartGameRequest,
) -> Result<GameStateResponse, AppError> {
    let game = find_game_by_admin_code(&state.pool, admin_code)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::GameNotFound)?;
    validate_admin_token(state, game.id, token).await?;

    match state.sessions.state(game.id).await {
        Ok(_) => return Err(AppError::GameAlreadyStarted),
        Err(SessionError::NotFound) => {}
        Err(err) => return Err(session_error(err)),
    }

    let players = players::repository::list_players_for_game(&state.pool, game.id)
        .await
        .map_err(AppError::Database)?;
    let pack = state
        .question_packs
        .load(&request.question_pack_id)
        .map_err(AppError::QuestionPack)?;
    let scenario = build_scenario(&pack, &players).map_err(AppError::QuestionPack)?;
    let engine =
        JeopardyGame::new(scenario).map_err(|err| AppError::Gameplay(format!("{err:?}")))?;

    repository::mark_game_started(&state.pool, game.id, pack.id.clone())
        .await
        .map_err(AppError::Database)?;

    state
        .sessions
        .create(RuntimeSession::new(
            game.id,
            pack.id.clone(),
            pack.clone(),
            players,
            engine,
        ))
        .await
        .map_err(session_error)?;

    realtime::notify(state, game.id, UpdateKind::State).await;
    info!(
        game_id = game.id,
        question_pack_id = %pack.id,
        "game started"
    );

    game_state_by_admin_code(state, admin_code).await
}

pub(crate) async fn game_state_by_admin_code(
    state: &AppState,
    admin_code: i32,
) -> Result<GameStateResponse, AppError> {
    let game = find_game_by_admin_code(&state.pool, admin_code)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::GameNotFound)?;
    game_state_by_game_id(state, game.id, true).await
}

pub(crate) async fn game_state_by_player_code(
    state: &AppState,
    player_code: i32,
) -> Result<GameStateResponse, AppError> {
    let game = find_game_by_player_code(&state.pool, player_code)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::GameNotFound)?;
    game_state_by_game_id(state, game.id, false).await
}

pub(crate) async fn submit_player_answer(
    state: &AppState,
    player_code: i32,
    request: PlayerAnswerRequest,
) -> Result<GameStateResponse, AppError> {
    let game = find_game_by_player_code(&state.pool, player_code)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::GameNotFound)?;

    state
        .sessions
        .submit_answer(game.id, request.player_id, request.answer)
        .await
        .map_err(session_error)?;

    realtime::notify(state, game.id, UpdateKind::State).await;
    info!(
        game_id = game.id,
        player_id = request.player_id,
        "player answer submission received"
    );

    game_state_by_game_id(state, game.id, false).await
}

pub(crate) async fn select_clue(
    state: &AppState,
    admin_code: i32,
    token: Option<&str>,
    request: SelectClueRequest,
) -> Result<GameStateResponse, AppError> {
    let game = authenticated_admin_game(state, admin_code, token).await?;
    let current_selector = state
        .sessions
        .state(game.id)
        .await
        .map_err(session_error)?
        .current_selector;
    apply_action(
        state,
        game.id,
        GameAction::SelectClue {
            player_id: current_selector,
            category_index: request.category_index,
            clue_index: request.clue_index,
        },
    )
    .await
}

pub(crate) async fn answer_clue(
    state: &AppState,
    admin_code: i32,
    token: Option<&str>,
    request: AnswerRequest,
) -> Result<GameStateResponse, AppError> {
    let game = authenticated_admin_game(state, admin_code, token).await?;
    apply_action(
        state,
        game.id,
        GameAction::AttemptAnswer {
            player_id: request.player_id,
            correct: request.correct,
        },
    )
    .await
}

pub(crate) async fn daily_double_wager(
    state: &AppState,
    admin_code: i32,
    token: Option<&str>,
    request: WagerRequest,
) -> Result<GameStateResponse, AppError> {
    let game = authenticated_admin_game(state, admin_code, token).await?;
    apply_action(
        state,
        game.id,
        GameAction::SubmitDailyDoubleWager {
            player_id: request.player_id,
            amount: request.amount,
        },
    )
    .await
}

pub(crate) async fn daily_double_resolve(
    state: &AppState,
    admin_code: i32,
    token: Option<&str>,
    request: ResolveRequest,
) -> Result<GameStateResponse, AppError> {
    let game = authenticated_admin_game(state, admin_code, token).await?;
    apply_action(
        state,
        game.id,
        GameAction::ResolveDailyDouble {
            player_id: request.player_id,
            correct: request.correct,
        },
    )
    .await
}

pub(crate) async fn final_wager(
    state: &AppState,
    admin_code: i32,
    token: Option<&str>,
    request: WagerRequest,
) -> Result<GameStateResponse, AppError> {
    let game = authenticated_admin_game(state, admin_code, token).await?;
    apply_action(
        state,
        game.id,
        GameAction::SubmitFinalWager {
            player_id: request.player_id,
            amount: request.amount,
        },
    )
    .await
}

pub(crate) async fn final_resolve(
    state: &AppState,
    admin_code: i32,
    token: Option<&str>,
    request: ResolveRequest,
) -> Result<GameStateResponse, AppError> {
    let game = authenticated_admin_game(state, admin_code, token).await?;
    apply_action(
        state,
        game.id,
        GameAction::ResolveFinalAnswer {
            player_id: request.player_id,
            correct: request.correct,
        },
    )
    .await
}

pub(crate) async fn finish_game(
    state: &AppState,
    admin_code: i32,
    token: Option<&str>,
) -> Result<FinishGameResponse, AppError> {
    let game = authenticated_admin_game(state, admin_code, token).await?;
    // Postgres keeps the durable completion record; Redis only loses the
    // transient runtime state.
    repository::mark_game_completed(&state.pool, game.id)
        .await
        .map_err(AppError::Database)?;
    state
        .sessions
        .remove(game.id)
        .await
        .map_err(session_error)?;

    realtime::notify(state, game.id, UpdateKind::Finished).await;
    info!(game_id = game.id, "game finished");

    Ok(FinishGameResponse { completed: true })
}

async fn apply_action(
    state: &AppState,
    game_id: i64,
    action: GameAction,
) -> Result<GameStateResponse, AppError> {
    let action_log = ActionLog::from(&action);
    let engine_state = state
        .sessions
        .apply(game_id, action)
        .await
        .map_err(session_error)?;
    action_log.emit(game_id);
    if engine_state.phase == GamePhase::Completed {
        repository::mark_game_completed(&state.pool, game_id)
            .await
            .map_err(AppError::Database)?;
        info!(game_id, "game completed through gameplay");
    }

    realtime::notify(state, game_id, UpdateKind::State).await;

    game_state_by_game_id(state, game_id, true).await
}

enum ActionLog {
    ClueSelected {
        player_id: u32,
        category_index: usize,
        clue_index: usize,
    },
    ClueResolved {
        player_id: u32,
        correct: bool,
    },
    DailyDoubleWagered {
        player_id: u32,
        amount: i32,
    },
    DailyDoubleResolved {
        player_id: u32,
        correct: bool,
    },
    FinalWagered {
        player_id: u32,
        amount: i32,
    },
    FinalResolved {
        player_id: u32,
        correct: bool,
    },
}

impl From<&GameAction> for ActionLog {
    fn from(action: &GameAction) -> Self {
        match action {
            GameAction::SelectClue {
                player_id,
                category_index,
                clue_index,
            } => Self::ClueSelected {
                player_id: *player_id,
                category_index: *category_index,
                clue_index: *clue_index,
            },
            GameAction::AttemptAnswer { player_id, correct } => Self::ClueResolved {
                player_id: *player_id,
                correct: *correct,
            },
            GameAction::SubmitDailyDoubleWager { player_id, amount } => Self::DailyDoubleWagered {
                player_id: *player_id,
                amount: *amount,
            },
            GameAction::ResolveDailyDouble { player_id, correct } => Self::DailyDoubleResolved {
                player_id: *player_id,
                correct: *correct,
            },
            GameAction::SubmitFinalWager { player_id, amount } => Self::FinalWagered {
                player_id: *player_id,
                amount: *amount,
            },
            GameAction::ResolveFinalAnswer { player_id, correct } => Self::FinalResolved {
                player_id: *player_id,
                correct: *correct,
            },
        }
    }
}

impl ActionLog {
    fn emit(self, game_id: i64) {
        match self {
            Self::ClueSelected {
                player_id,
                category_index,
                clue_index,
            } => info!(
                game_id,
                player_id, category_index, clue_index, "clue selected"
            ),
            Self::ClueResolved { player_id, correct } => {
                info!(game_id, player_id, correct, "clue answer resolved")
            }
            Self::DailyDoubleWagered { player_id, amount } => {
                info!(game_id, player_id, amount, "daily double wager submitted")
            }
            Self::DailyDoubleResolved { player_id, correct } => {
                info!(game_id, player_id, correct, "daily double resolved")
            }
            Self::FinalWagered { player_id, amount } => {
                info!(game_id, player_id, amount, "final wager submitted")
            }
            Self::FinalResolved { player_id, correct } => {
                info!(game_id, player_id, correct, "final answer resolved")
            }
        }
    }
}

async fn game_state_by_game_id(
    state: &AppState,
    game_id: i64,
    include_answers: bool,
) -> Result<GameStateResponse, AppError> {
    let session = state.sessions.get(game_id).await.map_err(session_error)?;
    Ok(GameStateResponse {
        game: build_game_view(
            session.state(),
            session.pack(),
            include_answers,
            session.submissions(),
        ),
    })
}

pub(crate) async fn authenticated_admin_game(
    state: &AppState,
    admin_code: i32,
    token: Option<&str>,
) -> Result<crate::games::models::GameRow, AppError> {
    let game = find_game_by_admin_code(&state.pool, admin_code)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::GameNotFound)?;
    validate_admin_token(state, game.id, token).await?;
    Ok(game)
}

async fn validate_admin_token(
    state: &AppState,
    game_id: i64,
    token: Option<&str>,
) -> Result<(), AppError> {
    let token = token.ok_or(AppError::MissingAdminToken)?;
    let token_hash = auth::hash_admin_token(token);
    let row = repository::find_active_admin_token_by_hash(&state.pool, &token_hash)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::InvalidAdminToken)?;

    if row.game_id != game_id {
        return Err(AppError::WrongGameForToken);
    }

    Ok(())
}

fn session_error(err: SessionError) -> AppError {
    match err {
        SessionError::NotFound => AppError::SessionNotFound,
        SessionError::Game(err) => AppError::Gameplay(format!("{err:?}")),
        SessionError::InvalidSubmission(err) => AppError::Gameplay(err),
        SessionError::Storage(err) => AppError::SessionStorage(err),
    }
}
