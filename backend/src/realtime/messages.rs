use serde::{Deserialize, Serialize};

use crate::{games::state::GameView, players::models::PlayerSummary};

/// Messages pushed to connected websocket clients. Tagged so the frontend can
/// validate them with a zod discriminated union.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Current lobby roster (pre-game and on join changes).
    Lobby { players: Vec<PlayerSummary> },
    /// Latest full game view for this audience.
    GameState { game: GameView },
    /// The host finished the game; the runtime session is gone.
    GameFinished,
    /// Reply to a client heartbeat.
    Pong,
    /// Connection-level problem (for example failed authentication).
    Error { message: String },
}

/// Messages accepted from websocket clients. Gameplay commands stay on REST;
/// the socket only carries heartbeats for now.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Ping,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_messages_serialize_with_type_tags() {
        let lobby = serde_json::to_value(ServerMessage::Lobby {
            players: vec![PlayerSummary {
                id: 1,
                display_name: "Ada".to_owned(),
            }],
        })
        .expect("lobby message should serialize");
        assert_eq!(lobby["type"], "lobby");
        assert_eq!(lobby["players"][0]["display_name"], "Ada");

        let finished = serde_json::to_value(ServerMessage::GameFinished)
            .expect("finished message should serialize");
        assert_eq!(finished["type"], "game_finished");

        let pong = serde_json::to_value(ServerMessage::Pong).expect("pong should serialize");
        assert_eq!(pong["type"], "pong");

        let error = serde_json::to_value(ServerMessage::Error {
            message: "invalid_admin_token".to_owned(),
        })
        .expect("error message should serialize");
        assert_eq!(error["type"], "error");
        assert_eq!(error["message"], "invalid_admin_token");
    }

    #[test]
    fn client_heartbeats_deserialize() {
        let message: ClientMessage =
            serde_json::from_str(r#"{"type":"ping"}"#).expect("ping should parse");
        assert!(matches!(message, ClientMessage::Ping));
    }
}
