use async_trait::async_trait;

use crate::{
    domain::jeopardy::{GameAction, GameError, GameState},
    sessions::runtime::RuntimeSession,
};

/// How long an abandoned session survives before the store evicts it.
/// Every save refreshes the TTL, so only games nobody touches expire.
pub const SESSION_TTL_SECONDS: u64 = 12 * 60 * 60;

/// Stable Redis key for a game's runtime session.
pub fn session_key(game_id: i64) -> String {
    format!("jeopardy:game:{game_id}:session")
}

#[derive(Debug)]
pub enum SessionError {
    NotFound,
    Game(GameError),
    InvalidSubmission(String),
    Storage(String),
}

/// Persistence for active runtime sessions. PostgreSQL stays the durable
/// source of truth for game records; this store only holds the transient
/// engine state needed for live gameplay.
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Persist a brand-new session for a game.
    async fn create(&self, session: RuntimeSession) -> Result<(), SessionError>;

    /// Load the full runtime session (with-session style access on an owned copy).
    async fn get(&self, game_id: i64) -> Result<RuntimeSession, SessionError>;

    /// Persist an updated session.
    async fn save(&self, session: &RuntimeSession) -> Result<(), SessionError>;

    /// Remove the session, returning it if it existed.
    async fn remove(&self, game_id: i64) -> Result<Option<RuntimeSession>, SessionError>;

    /// Load just the engine state.
    async fn state(&self, game_id: i64) -> Result<GameState, SessionError> {
        Ok(self.get(game_id).await?.state().clone())
    }

    /// Apply a gameplay action and persist the result.
    async fn apply(&self, game_id: i64, action: GameAction) -> Result<GameState, SessionError> {
        let mut session = self.get(game_id).await?;
        session.apply(action).map_err(SessionError::Game)?;
        self.save(&session).await?;
        Ok(session.state().clone())
    }

    /// Record a player's free-text answer and persist the result.
    async fn submit_answer(
        &self,
        game_id: i64,
        player_id: u32,
        answer: String,
    ) -> Result<(), SessionError> {
        let mut session = self.get(game_id).await?;
        session
            .submit_answer(player_id, answer)
            .map_err(SessionError::InvalidSubmission)?;
        self.save(&session).await
    }
}
