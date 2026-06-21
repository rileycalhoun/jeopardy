use serde::{Deserialize, Serialize};

use crate::{
    content::models::CategorySummary, games::state::GameView, players::models::PlayerSummary,
};

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct GameData {
    pub game_id: String,
    pub admin_code: i32,
    pub player_code: i32,
}

#[derive(Serialize)]
pub struct CreateGameResponse {
    pub admin_code: i32,
    pub player_code: i32,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct GameRow {
    pub id: i64,
    pub game_id: String,
    pub admin_code: i32,
    pub player_code: i32,
}

#[derive(Deserialize)]
pub struct JoinGameRequest {
    pub player_code: i32,
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct JoinAdminRequest {
    pub admin_code: i32,
}

#[derive(Serialize)]
pub struct LobbyResponse {
    pub players: Vec<PlayerSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_player_id: Option<i64>,
}

#[derive(Serialize)]
pub struct CategoriesResponse {
    pub categories: Vec<CategorySummary>,
}

#[derive(Deserialize)]
pub struct StartGameRequest {
    pub category_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct GameStateResponse {
    pub game: GameView,
}

#[derive(Deserialize)]
pub struct SelectClueRequest {
    pub category_index: usize,
    pub clue_index: usize,
}

#[derive(Deserialize)]
pub struct PlayerSelectClueRequest {
    pub player_id: u32,
    pub category_index: usize,
    pub clue_index: usize,
}

#[derive(Deserialize)]
pub struct AnswerRequest {
    pub player_id: u32,
    pub correct: bool,
}

#[derive(Deserialize)]
pub struct PlayerAnswerRequest {
    pub player_id: u32,
    pub answer: String,
}

#[derive(Deserialize)]
pub struct WagerRequest {
    pub player_id: u32,
    pub amount: i32,
}

#[derive(Deserialize)]
pub struct ResolveRequest {
    pub player_id: u32,
    pub correct: bool,
}

#[derive(Serialize)]
pub struct FinishGameResponse {
    pub completed: bool,
}
