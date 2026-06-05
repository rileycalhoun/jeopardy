use serde::Serialize;

#[derive(Serialize)]
pub struct PlayerRow {
    pub id: i64,
    pub game_id: i64,
    pub display_name: String,
}

#[derive(Serialize)]
pub struct PlayerSummary {
    pub id: i64,
    pub display_name: String,
}
