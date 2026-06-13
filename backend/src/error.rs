use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Database(sqlx::Error),
    GameCreationExhausted,
    GameNotFound,
    DuplicatePlayerName,
    QuestionPack(String),
    SessionNotFound,
    SessionStorage(String),
    Gameplay(String),
    NotYourTurn,
    MissingAdminToken,
    InvalidAdminToken,
    WrongGameForToken,
    GameAlreadyStarted,
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, unexpected) = match &self {
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_server_error",
                true,
            ),
            AppError::GameCreationExhausted => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "game_creation_failed",
                true,
            ),
            AppError::GameNotFound => (StatusCode::NOT_FOUND, "game_not_found", false),
            AppError::DuplicatePlayerName => (StatusCode::CONFLICT, "duplicate_player_name", false),
            AppError::QuestionPack(_) => (StatusCode::BAD_REQUEST, "question_pack_error", false),
            AppError::SessionNotFound => (StatusCode::NOT_FOUND, "game_session_not_found", false),
            AppError::SessionStorage(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "session_storage_error",
                true,
            ),
            AppError::Gameplay(_) => (StatusCode::BAD_REQUEST, "invalid_game_action", false),
            AppError::NotYourTurn => (StatusCode::FORBIDDEN, "not_your_turn", false),
            AppError::MissingAdminToken => (StatusCode::UNAUTHORIZED, "missing_admin_token", false),
            AppError::InvalidAdminToken => (StatusCode::UNAUTHORIZED, "invalid_admin_token", false),
            AppError::WrongGameForToken => (StatusCode::FORBIDDEN, "wrong_game_for_token", false),
            AppError::GameAlreadyStarted => (StatusCode::CONFLICT, "game_already_started", false),
        };

        if unexpected {
            tracing::error!(error_code = code, error = ?self, "request failed");
        } else {
            tracing::warn!(error_code = code, error = ?self, "request rejected");
        }

        (status, Json(ErrorBody { error: code })).into_response()
    }
}
