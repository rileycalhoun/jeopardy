use std::{collections::HashMap, sync::Arc};

use tokio::sync::{RwLock, broadcast};
use tracing::{debug, warn};

use crate::{games::state::GameView, players::models::PlayerSummary};

/// One update fanned out to every socket watching a game. Connections pick
/// the view that matches their audience (admins see answers, players do not).
#[derive(Clone, Debug, Default)]
pub struct GameBroadcast {
    pub lobby: Option<Vec<PlayerSummary>>,
    pub admin_game: Option<GameView>,
    pub player_game: Option<GameView>,
    pub finished: bool,
}

const CHANNEL_CAPACITY: usize = 32;

/// In-process fan-out of game updates to connected websocket clients.
/// Cross-instance delivery happens through Redis pub/sub, which feeds
/// each instance's hub.
#[derive(Clone, Default)]
pub struct Hub {
    channels: Arc<RwLock<HashMap<i64, broadcast::Sender<Arc<GameBroadcast>>>>>,
}

impl Hub {
    pub async fn subscribe(&self, game_id: i64) -> broadcast::Receiver<Arc<GameBroadcast>> {
        let mut channels = self.channels.write().await;
        channels
            .entry(game_id)
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0)
            .subscribe()
    }

    pub async fn publish(&self, game_id: i64, update: GameBroadcast) {
        let mut channels = self.channels.write().await;
        let Some(sender) = channels.get(&game_id) else {
            debug!(
                game_id,
                "no websocket clients subscribed; skipping broadcast"
            );
            return;
        };

        match sender.send(Arc::new(update)) {
            Ok(receivers) => debug!(game_id, receivers, "broadcast game update"),
            Err(err) => {
                // All receivers disconnected since the channel was created.
                warn!(game_id, "broadcast failed, pruning channel: {err}");
                channels.remove(&game_id);
            }
        }
    }
}
