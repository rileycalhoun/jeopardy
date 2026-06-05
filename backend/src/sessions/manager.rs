use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    domain::jeopardy::{GameAction, GameError, GameState},
    sessions::runtime::RuntimeSession,
};

#[derive(Clone, Default)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<i64, RuntimeSession>>>,
}

#[derive(Debug)]
pub enum SessionError {
    NotFound,
    Game(GameError),
    InvalidSubmission(String),
}

impl SessionManager {
    pub async fn create(&self, session: RuntimeSession) {
        self.sessions.write().await.insert(session.game_id, session);
    }

    pub async fn state(&self, game_id: i64) -> Result<GameState, SessionError> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(&game_id).ok_or(SessionError::NotFound)?;
        Ok(session.state().clone())
    }

    pub async fn with_session<T>(
        &self,
        game_id: i64,
        f: impl FnOnce(&RuntimeSession) -> T,
    ) -> Result<T, SessionError> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(&game_id).ok_or(SessionError::NotFound)?;
        Ok(f(session))
    }

    pub async fn apply(&self, game_id: i64, action: GameAction) -> Result<GameState, SessionError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(&game_id).ok_or(SessionError::NotFound)?;
        session.apply(action).map_err(SessionError::Game)?;
        Ok(session.state().clone())
    }

    pub async fn submit_answer(
        &self,
        game_id: i64,
        player_id: u32,
        answer: String,
    ) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(&game_id).ok_or(SessionError::NotFound)?;
        session
            .submit_answer(player_id, answer)
            .map_err(SessionError::InvalidSubmission)?;
        Ok(())
    }

    pub async fn remove(&self, game_id: i64) -> Option<RuntimeSession> {
        self.sessions.write().await.remove(&game_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        content::models::QuestionPack,
        domain::jeopardy::{
            Category, Clue, GameAction, GameScenario, JeopardyGame, PlayerState, RoundBoard,
        },
        players::models::PlayerSummary,
        sessions::runtime::RuntimeSession,
    };

    use super::*;

    #[tokio::test]
    async fn stores_runtime_sessions_in_memory() {
        let manager = SessionManager::default();
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
            starting_selector: 7,
        })
        .expect("scenario should be valid");

        manager
            .create(RuntimeSession::new(
                99,
                "classic".to_owned(),
                QuestionPack {
                    id: "classic".to_owned(),
                    title: "Classic".to_owned(),
                    rounds: Vec::new(),
                    final_jeopardy: None,
                },
                vec![PlayerSummary {
                    id: 7,
                    display_name: "Ada".to_owned(),
                }],
                game,
            ))
            .await;

        let state = manager.state(99).await.expect("session should exist");
        assert_eq!(state.players[0].score, 0);

        let state = manager
            .apply(
                99,
                GameAction::SelectClue {
                    player_id: 7,
                    category_index: 0,
                    clue_index: 0,
                },
            )
            .await
            .expect("action should apply");

        assert!(state.active_clue.is_some());

        manager
            .submit_answer(99, 7, "What is let?".to_owned())
            .await
            .expect("submission should be accepted");

        let submissions = manager
            .with_session(99, |session| session.submissions().to_vec())
            .await
            .expect("session should exist");

        assert_eq!(submissions[0].player_id, 7);
        assert_eq!(submissions[0].player_name, "Ada");
        assert_eq!(submissions[0].answer, "What is let?");
    }
}
