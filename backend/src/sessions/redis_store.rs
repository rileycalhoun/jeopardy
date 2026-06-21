use async_trait::async_trait;
use redis::{AsyncCommands, aio::ConnectionManager};
use tracing::debug;

use crate::sessions::{
    runtime::RuntimeSession,
    store::{SESSION_TTL_SECONDS, SessionError, SessionStore, session_key},
};

/// Redis-backed session store. Sessions live under
/// `jeopardy:game:{game_id}:session` as JSON with a refresh-on-write TTL, so
/// active gameplay survives backend restarts and works across instances.
#[derive(Clone)]
pub struct RedisSessionStore {
    conn: ConnectionManager,
}

impl RedisSessionStore {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }

    fn storage_error(context: &str, err: impl std::fmt::Display) -> SessionError {
        SessionError::Storage(format!("{context}: {err}"))
    }
}

#[async_trait]
impl SessionStore for RedisSessionStore {
    async fn create(&self, session: RuntimeSession) -> Result<(), SessionError> {
        self.save(&session).await
    }

    async fn get(&self, game_id: i64) -> Result<RuntimeSession, SessionError> {
        let key = session_key(game_id);
        let mut conn = self.conn.clone();
        let raw: Option<String> = conn
            .get(&key)
            .await
            .map_err(|err| Self::storage_error("load", err))?;
        let raw = raw.ok_or(SessionError::NotFound)?;
        debug!(game_id, "loaded session from redis");
        serde_json::from_str(&raw).map_err(|err| Self::storage_error("decode", err))
    }

    async fn save(&self, session: &RuntimeSession) -> Result<(), SessionError> {
        let key = session_key(session.game_id);
        let json =
            serde_json::to_string(session).map_err(|err| Self::storage_error("encode", err))?;
        let mut conn = self.conn.clone();
        conn.set_ex::<_, _, ()>(&key, json, SESSION_TTL_SECONDS)
            .await
            .map_err(|err| Self::storage_error("save", err))?;
        debug!(game_id = session.game_id, "saved session to redis");
        Ok(())
    }

    async fn remove(&self, game_id: i64) -> Result<Option<RuntimeSession>, SessionError> {
        let session = match self.get(game_id).await {
            Ok(session) => Some(session),
            Err(SessionError::NotFound) => None,
            Err(err) => return Err(err),
        };
        let key = session_key(game_id);
        let mut conn = self.conn.clone();
        conn.del::<_, ()>(&key)
            .await
            .map_err(|err| Self::storage_error("remove", err))?;
        debug!(game_id, "removed session from redis");
        Ok(session)
    }
}

// Integration tests that need a live Redis. Run them explicitly with:
//   REDIS_URL=redis://127.0.0.1:6379 cargo test --lib redis_store -- --ignored
#[cfg(test)]
mod tests {
    use crate::{
        content::models::{BoardCategory, BoardClue, BoardContent, BoardRound},
        domain::jeopardy::{
            Category, Clue, GameAction, GameScenario, JeopardyGame, PlayerState, RoundBoard,
            Selector,
        },
        players::models::PlayerSummary,
        sessions::runtime::RuntimeSession,
    };

    use super::*;

    fn sample_session(game_id: i64) -> RuntimeSession {
        let game = JeopardyGame::new(GameScenario {
            players: vec![PlayerState {
                id: 7,
                name: "Ada".to_owned(),
                score: 0,
                final_wager: None,
                final_answer_correct: None,
            }],
            rounds: vec![RoundBoard {
                name: "Jeopardy".to_owned(),
                categories: vec![Category {
                    title: "Rust".to_owned(),
                    clues: vec![Clue {
                        id: 1,
                        label: "$200".to_owned(),
                        value: 200,
                        is_daily_double: false,
                        answered: false,
                    }],
                }],
            }],
            final_jeopardy: None,
        })
        .expect("scenario should be valid");

        RuntimeSession::new(
            game_id,
            "classic".to_owned(),
            BoardContent {
                id: "classic".to_owned(),
                title: "Classic".to_owned(),
                rounds: vec![BoardRound {
                    name: "Jeopardy".to_owned(),
                    categories: vec![BoardCategory {
                        id: "rust".to_owned(),
                        title: "Rust".to_owned(),
                        clues: vec![BoardClue {
                            label: "$200".to_owned(),
                            question: "This keyword creates an immutable binding.".to_owned(),
                            answer: "What is let?".to_owned(),
                            value: 200,
                            daily_double: false,
                        }],
                    }],
                }],
                final_jeopardy: None,
            },
            vec![PlayerSummary {
                id: 7,
                display_name: "Ada".to_owned(),
            }],
            game,
        )
    }

    async fn connect() -> RedisSessionStore {
        let url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_owned());
        let client = redis::Client::open(url).expect("redis url should parse");
        let conn = ConnectionManager::new(client)
            .await
            .expect("redis should be reachable");
        RedisSessionStore::new(conn)
    }

    #[tokio::test]
    #[ignore = "requires a running Redis instance"]
    async fn sessions_round_trip_through_redis() {
        let store = connect().await;
        // Use a distinctive id so the test does not collide with real games.
        let game_id = -9_900_001;
        let _ = store.remove(game_id).await;

        store
            .create(sample_session(game_id))
            .await
            .expect("create should succeed");

        let state = store
            .apply(
                game_id,
                GameAction::SelectClue {
                    actor: Selector::Moderator,
                    category_index: 0,
                    clue_index: 0,
                },
            )
            .await
            .expect("action should apply");
        assert!(state.active_clue.is_some());

        store
            .submit_answer(game_id, 7, "What is let?".to_owned())
            .await
            .expect("submission should be accepted");

        let session = store.get(game_id).await.expect("session should exist");
        assert_eq!(session.submissions()[0].answer, "What is let?");

        let removed = store.remove(game_id).await.expect("remove should succeed");
        assert!(removed.is_some());
        assert!(matches!(
            store.get(game_id).await,
            Err(SessionError::NotFound)
        ));
    }
}
