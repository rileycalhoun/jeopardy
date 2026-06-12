pub mod hub;
pub mod messages;
pub mod ws;

use std::time::Duration;

use futures_util::StreamExt;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::{
    games::state::build_game_view, players, realtime::hub::GameBroadcast, state::AppState,
};

/// Redis pub/sub channel that fans game update events out to every backend
/// instance, so websocket clients get updates no matter which instance
/// handled the REST mutation.
pub const EVENTS_CHANNEL: &str = "jeopardy:events";

/// What changed about a game. The receiving instance loads the current state
/// itself (from Postgres for the lobby, Redis for the session), so events
/// stay small and stale events are harmless.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateKind {
    Lobby,
    State,
    Finished,
}

#[derive(Serialize, Deserialize)]
struct EventEnvelope {
    game_id: i64,
    kind: UpdateKind,
    origin: String,
}

/// Broadcast an update to local websocket clients and publish it for other
/// backend instances. Call this after every successful gameplay mutation.
pub async fn notify(state: &AppState, game_id: i64, kind: UpdateKind) {
    broadcast_to_local_clients(state, game_id, kind).await;

    let envelope = EventEnvelope {
        game_id,
        kind,
        origin: state.instance_id.clone(),
    };
    let payload = match serde_json::to_string(&envelope) {
        Ok(payload) => payload,
        Err(err) => {
            warn!(game_id, ?err, "could not encode update event");
            return;
        }
    };

    let mut conn = state.redis.clone();
    if let Err(err) = conn.publish::<_, _, ()>(EVENTS_CHANNEL, payload).await {
        warn!(game_id, ?err, "could not publish update event to redis");
    }
}

/// Build the latest views for a game and hand them to the local hub.
async fn broadcast_to_local_clients(state: &AppState, game_id: i64, kind: UpdateKind) {
    let mut update = GameBroadcast::default();

    match kind {
        UpdateKind::Lobby => {
            match players::repository::list_players_for_game(&state.pool, game_id).await {
                Ok(players) => update.lobby = Some(players),
                Err(err) => {
                    warn!(game_id, ?err, "could not load lobby for broadcast");
                    return;
                }
            }
        }
        UpdateKind::State => match state.sessions.get(game_id).await {
            Ok(session) => {
                update.admin_game = Some(build_game_view(
                    session.state(),
                    session.pack(),
                    true,
                    session.submissions(),
                ));
                update.player_game = Some(build_game_view(
                    session.state(),
                    session.pack(),
                    false,
                    session.submissions(),
                ));
            }
            Err(err) => {
                warn!(game_id, ?err, "could not load session for broadcast");
                return;
            }
        },
        UpdateKind::Finished => update.finished = true,
    }

    state.hub.publish(game_id, update).await;
}

/// Subscribe to the Redis events channel and rebroadcast updates that
/// originated on other backend instances. Reconnects with a fixed delay.
pub fn spawn_event_listener(state: AppState, client: redis::Client) {
    info!(channel = EVENTS_CHANNEL, "starting redis pubsub listener");
    tokio::spawn(async move {
        loop {
            match client.get_async_pubsub().await {
                Ok(mut pubsub) => {
                    if let Err(err) = pubsub.subscribe(EVENTS_CHANNEL).await {
                        warn!(?err, "could not subscribe to redis events channel");
                    } else {
                        info!(
                            channel = EVENTS_CHANNEL,
                            "subscribed to redis update events"
                        );
                        let mut stream = pubsub.on_message();
                        while let Some(message) = stream.next().await {
                            let payload: String = match message.get_payload() {
                                Ok(payload) => payload,
                                Err(err) => {
                                    warn!(?err, "could not read update event payload");
                                    continue;
                                }
                            };
                            let envelope: EventEnvelope = match serde_json::from_str(&payload) {
                                Ok(envelope) => envelope,
                                Err(err) => {
                                    warn!(?err, "could not decode update event");
                                    continue;
                                }
                            };
                            if envelope.origin == state.instance_id {
                                // Already broadcast locally when the mutation ran.
                                continue;
                            }
                            debug!(
                                game_id = envelope.game_id,
                                kind = ?envelope.kind,
                                "received update event from another instance"
                            );
                            broadcast_to_local_clients(&state, envelope.game_id, envelope.kind)
                                .await;
                        }
                        warn!("redis pubsub stream ended; reconnecting");
                    }
                }
                Err(err) => warn!(?err, "could not open redis pubsub connection"),
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}
