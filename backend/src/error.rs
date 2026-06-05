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
        }
    }
}
