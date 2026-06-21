use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::sessions::{
    runtime::RuntimeSession,
    store::{SessionError, SessionStore},
};

/// In-process session store. Used by unit tests (and available as a fallback
/// when Redis is intentionally absent); production uses `RedisSessionStore`.
#[derive(Clone, Default)]
pub struct MemorySessionStore {
    sessions: Arc<RwLock<HashMap<i64, RuntimeSession>>>,
}

#[async_trait]
impl SessionStore for MemorySessionStore {
    async fn create(&self, session: RuntimeSession) -> Result<(), SessionError> {
        self.sessions.write().await.insert(session.game_id, session);
        Ok(())
    }

    async fn get(&self, game_id: i64) -> Result<RuntimeSession, SessionError> {
        self.sessions
            .read()
            .await
            .get(&game_id)
            .cloned()
            .ok_or(SessionError::NotFound)
    }

    async fn save(&self, session: &RuntimeSession) -> Result<(), SessionError> {
        self.sessions
            .write()
            .await
            .insert(session.game_id, session.clone());
        Ok(())
    }

    async fn remove(&self, game_id: i64) -> Result<Option<RuntimeSession>, SessionError> {
        Ok(self.sessions.write().await.remove(&game_id))
    }
}

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

    #[tokio::test]
    async fn stores_runtime_sessions() {
        let store = MemorySessionStore::default();
        store
            .create(sample_session(99))
            .await
            .expect("create should succeed");

        let state = store.state(99).await.expect("session should exist");
        assert_eq!(state.players[0].score, 0);

        let state = store
            .apply(
                99,
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
            .submit_answer(99, 7, "What is let?".to_owned())
            .await
            .expect("submission should be accepted");

        let session = store.get(99).await.expect("session should exist");
        let submissions = session.submissions();

        assert_eq!(submissions[0].player_id, 7);
        assert_eq!(submissions[0].player_name, "Ada");
        assert_eq!(submissions[0].answer, "What is let?");
    }

    #[tokio::test]
    async fn remove_returns_the_stored_session() {
        let store = MemorySessionStore::default();
        store
            .create(sample_session(42))
            .await
            .expect("create should succeed");

        let removed = store.remove(42).await.expect("remove should succeed");
        assert!(removed.is_some());
        assert!(matches!(store.get(42).await, Err(SessionError::NotFound)));
    }

    #[tokio::test]
    async fn sessions_round_trip_through_serde() {
        let session = sample_session(7);
        let json = serde_json::to_string(&session).expect("session should serialize");
        let decoded: RuntimeSession =
            serde_json::from_str(&json).expect("session should deserialize");

        assert_eq!(decoded.game_id, 7);
        assert_eq!(decoded.content_id, "classic");
        assert_eq!(decoded.state(), session.state());
        assert_eq!(decoded.roster().len(), 1);
    }
}
