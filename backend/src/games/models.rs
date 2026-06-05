use serde::{Deserialize, Serialize};

use crate::players::models::PlayerSummary;

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize)]
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
}
