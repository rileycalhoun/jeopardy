use std::time::Duration;

use axum::{
    extract::{
        Extension, Path, Query, State,
        ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use serde::Deserialize;
use tokio::time::Instant;
use tower_http::request_id::RequestId;
use tracing::{debug, error, info, warn};

use crate::{
    error::AppError,
    games::{
        repository::find_game_by_player_code,
        service,
        state::{GameView, build_game_view},
    },
    players,
    realtime::{
        hub::GameBroadcast,
        messages::{ClientMessage, ServerMessage},
    },
    sessions::store::SessionError,
    state::AppState,
};

/// Server-initiated ping cadence and how long a silent client survives.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(90);

/// Close code for connections we reject after the upgrade (auth failures).
const CLOSE_UNAUTHORIZED: u16 = 4401;

#[derive(Clone, Copy, Debug)]
enum Audience {
    Admin,
    Player,
}

#[derive(Deserialize)]
pub struct AdminSocketQuery {
    token: Option<String>,
}

/// GET /ws/games/admin/{admin_code}?token=...
pub async fn admin_socket(
    State(state): State<AppState>,
    Path(admin_code): Path<i32>,
    Query(query): Query<AdminSocketQuery>,
    Extension(request_id): Extension<RequestId>,
    ws: WebSocketUpgrade,
) -> Response {
    let request_id = request_id
        .header_value()
        .to_str()
        .unwrap_or("<invalid>")
        .to_owned();
    ws.on_upgrade(move |socket| async move {
        let game =
            match service::authenticated_admin_game(&state, admin_code, query.token.as_deref())
                .await
            {
                Ok(game) => game,
                Err(err) => {
                    let reason = auth_error_code(&err);
                    if reason == "internal_error" {
                        error!(request_id, ?err, "rejected admin websocket");
                    } else {
                        warn!(request_id, reason, "rejected admin websocket");
                    }
                    reject(socket, reason).await;
                    return;
                }
            };

        info!(request_id, game_id = game.id, "admin websocket connected");
        run_connection(state, socket, game.id, Audience::Admin, &request_id).await;
    })
}

/// GET /ws/games/player/{player_code}
pub async fn player_socket(
    State(state): State<AppState>,
    Path(player_code): Path<i32>,
    Extension(request_id): Extension<RequestId>,
    ws: WebSocketUpgrade,
) -> Response {
    let request_id = request_id
        .header_value()
        .to_str()
        .unwrap_or("<invalid>")
        .to_owned();
    ws.on_upgrade(move |socket| async move {
        let game = match find_game_by_player_code(&state.pool, player_code).await {
            Ok(Some(game)) => game,
            Ok(None) => {
                warn!(
                    request_id,
                    reason = "game_not_found",
                    "rejected player websocket"
                );
                reject(socket, "game_not_found").await;
                return;
            }
            Err(err) => {
                error!(request_id, ?err, "rejected player websocket");
                reject(socket, "internal_error").await;
                return;
            }
        };

        info!(request_id, game_id = game.id, "player websocket connected");
        run_connection(state, socket, game.id, Audience::Player, &request_id).await;
    })
}

/// Send a useful error message, then close cleanly.
async fn reject(mut socket: WebSocket, reason: &'static str) {
    let _ = send_json(
        &mut socket,
        &ServerMessage::Error {
            message: reason.to_owned(),
        },
    )
    .await;
    let _ = socket
        .send(Message::Close(Some(CloseFrame {
            code: CLOSE_UNAUTHORIZED,
            reason: reason.into(),
        })))
        .await;
}

fn auth_error_code(err: &AppError) -> &'static str {
    match err {
        AppError::GameNotFound => "game_not_found",
        AppError::MissingAdminToken => "missing_admin_token",
        AppError::InvalidAdminToken => "invalid_admin_token",
        AppError::WrongGameForToken => "wrong_game_for_token",
        _ => "internal_error",
    }
}

async fn run_connection(
    state: AppState,
    mut socket: WebSocket,
    game_id: i64,
    audience: Audience,
    request_id: &str,
) {
    // Subscribe before the initial snapshot so updates racing the snapshot
    // are queued rather than lost.
    let mut updates = state.hub.subscribe(game_id).await;

    if send_initial_state(&state, &mut socket, game_id, audience)
        .await
        .is_err()
    {
        warn!(
            request_id,
            game_id,
            ?audience,
            "could not send initial websocket state"
        );
        return;
    }

    let mut heartbeat = tokio::time::interval(HEARTBEAT_INTERVAL);
    heartbeat.tick().await; // The first tick fires immediately; skip it.
    let mut last_seen = Instant::now();

    loop {
        tokio::select! {
            update = updates.recv() => match update {
                Ok(update) => {
                    for message in messages_for(&update, audience) {
                        if send_json(&mut socket, &message).await.is_err() {
                            info!(request_id, game_id, ?audience, "websocket send failed; disconnecting");
                            return;
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    // The client only needs the latest state; resync it.
                    warn!(request_id, game_id, skipped, "websocket lagged behind broadcasts; resyncing");
                    if send_initial_state(&state, &mut socket, game_id, audience).await.is_err() {
                        return;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            },
            frame = socket.recv() => match frame {
                Some(Ok(Message::Text(text))) => {
                    last_seen = Instant::now();
                    if let Ok(ClientMessage::Ping) = serde_json::from_str::<ClientMessage>(&text) {
                        if send_json(&mut socket, &ServerMessage::Pong).await.is_err() {
                            break;
                        }
                    }
                }
                Some(Ok(Message::Ping(_) | Message::Pong(_))) => {
                    last_seen = Instant::now();
                }
                Some(Ok(Message::Close(_))) | None => break,
                Some(Ok(Message::Binary(_))) => {}
                Some(Err(err)) => {
                    debug!(request_id, game_id, ?audience, ?err, "websocket receive error");
                    break;
                }
            },
            _ = heartbeat.tick() => {
                if last_seen.elapsed() > CLIENT_TIMEOUT {
                    info!(request_id, game_id, ?audience, "websocket timed out; disconnecting");
                    break;
                }
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() {
                    break;
                }
            },
        }
    }

    info!(request_id, game_id, ?audience, "websocket disconnected");
}

/// Push the current lobby and (if a session exists) game state on connect.
async fn send_initial_state(
    state: &AppState,
    socket: &mut WebSocket,
    game_id: i64,
    audience: Audience,
) -> Result<(), ()> {
    let players = players::repository::list_players_for_game(&state.pool, game_id)
        .await
        .map_err(|err| {
            error!(game_id, ?err, "could not load lobby for websocket");
        })?;
    send_json(socket, &ServerMessage::Lobby { players }).await?;

    match state.sessions.get(game_id).await {
        Ok(session) => {
            let include_answers = matches!(audience, Audience::Admin);
            let game = build_game_view(
                session.state(),
                session.pack(),
                include_answers,
                session.submissions(),
            );
            send_json(socket, &ServerMessage::GameState { game }).await?;
        }
        Err(SessionError::NotFound) => {
            debug!(game_id, "no active session for initial websocket state");
        }
        Err(err) => {
            error!(game_id, ?err, "could not load initial websocket state");
            return Err(());
        }
    }

    Ok(())
}

fn messages_for(update: &GameBroadcast, audience: Audience) -> Vec<ServerMessage> {
    let mut messages = Vec::new();

    if let Some(players) = &update.lobby {
        messages.push(ServerMessage::Lobby {
            players: players.clone(),
        });
    }

    let game: Option<&GameView> = match audience {
        Audience::Admin => update.admin_game.as_ref(),
        Audience::Player => update.player_game.as_ref(),
    };
    if let Some(game) = game {
        messages.push(ServerMessage::GameState { game: game.clone() });
    }

    if update.finished {
        messages.push(ServerMessage::GameFinished);
    }

    messages
}

async fn send_json(socket: &mut WebSocket, message: &ServerMessage) -> Result<(), ()> {
    let json = serde_json::to_string(message).map_err(|err| {
        warn!(?err, "could not encode websocket message");
    })?;
    socket
        .send(Message::Text(json.into()))
        .await
        .map_err(|_| ())
}
