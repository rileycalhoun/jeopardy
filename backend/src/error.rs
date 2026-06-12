use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

pub type AppResult<T> = Result<T, AppError>;

pub enum AppError {
    Database(sqlx::Error),
    GameCreationExhausted,
    GameNotFound,
    DuplicatePlayerName,
    QuestionPack(String),
    SessionNotFound,
    SessionStorage(String),
    Gameplay(String),
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
        match self {
            AppError::Database(err) => {
                tracing::error!("database error: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody {
                        error: "internal_server_error",
                    }),
                )
                    .into_response()
            }
            AppError::GameCreationExhausted => {
                tracing::error!("could not generate unique values after 10 tries");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody {
                        error: "game_creation_failed",
                    }),
                )
                    .into_response()
            }
            AppError::GameNotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorBody {
                    error: "game_not_found",
                }),
            )
                .into_response(),
            AppError::DuplicatePlayerName => (
                StatusCode::CONFLICT,
                Json(ErrorBody {
                    error: "duplicate_player_name",
                }),
            )
                .into_response(),
            AppError::QuestionPack(err) => {
                tracing::warn!("question pack error: {}", err);
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorBody {
                        error: "question_pack_error",
                    }),
                )
                    .into_response()
            }
            AppError::SessionNotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorBody {
                    error: "game_session_not_found",
                }),
            )
                .into_response(),
            AppError::SessionStorage(err) => {
                tracing::error!("session storage error: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody {
                        error: "session_storage_error",
                    }),
                )
                    .into_response()
            }
            AppError::Gameplay(err) => {
                tracing::warn!("gameplay error: {}", err);
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorBody {
                        error: "invalid_game_action",
                    }),
                )
                    .into_response()
            }
            AppError::MissingAdminToken => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody {
                    error: "missing_admin_token",
                }),
            )
                .into_response(),
            AppError::InvalidAdminToken => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody {
                    error: "invalid_admin_token",
                }),
            )
                .into_response(),
            AppError::WrongGameForToken => (
                StatusCode::FORBIDDEN,
                Json(ErrorBody {
                    error: "wrong_game_for_token",
                }),
            )
                .into_response(),
            AppError::GameAlreadyStarted => (
                StatusCode::CONFLICT,
                Json(ErrorBody {
                    error: "game_already_started",
                }),
            )
                .into_response(),
        }
    }
}
