use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::{
    content::models::BoardContent,
    domain::jeopardy::{GameAction, GameError, GameState, JeopardyGame},
    players::models::PlayerSummary,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnswerSubmission {
    pub player_id: u32,
    pub player_name: String,
    pub answer: String,
}

/// The full runtime state of an active game. Serialized to Redis on every
/// mutation so gameplay survives backend restarts and works across instances.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeSession {
    pub game_id: i64,
    pub content_id: String,
    board_content: BoardContent,
    roster: Vec<PlayerSummary>,
    game: JeopardyGame,
    submissions: Vec<AnswerSubmission>,
    pub updated_at: SystemTime,
}

impl RuntimeSession {
    pub fn new(
        game_id: i64,
        content_id: String,
        board_content: BoardContent,
        roster: Vec<PlayerSummary>,
        game: JeopardyGame,
    ) -> Self {
        Self {
            game_id,
            content_id,
            board_content,
            roster,
            game,
            submissions: Vec::new(),
            updated_at: SystemTime::now(),
        }
    }

    pub fn state(&self) -> &GameState {
        self.game.state()
    }

    pub fn board_content(&self) -> &BoardContent {
        &self.board_content
    }

    pub fn roster(&self) -> &[PlayerSummary] {
        &self.roster
    }

    pub fn apply(&mut self, action: GameAction) -> Result<(), GameError> {
        self.game.apply(action)?;
        if self.game.state().active_clue.is_none() {
            self.submissions.clear();
        }
        self.updated_at = SystemTime::now();
        Ok(())
    }

    pub fn submissions(&self) -> &[AnswerSubmission] {
        &self.submissions
    }

    pub fn submit_answer(&mut self, player_id: u32, answer: String) -> Result<(), String> {
        if self.game.state().active_clue.is_none() {
            return Err("answers can only be submitted while a clue is active".to_owned());
        }

        let answer = answer.trim().to_owned();
        if answer.is_empty() {
            return Err("answer must not be empty".to_owned());
        }

        let player = self
            .roster
            .iter()
            .find(|player| u32::try_from(player.id).ok() == Some(player_id))
            .ok_or_else(|| "player is not part of this game".to_owned())?;

        if let Some(existing) = self
            .submissions
            .iter_mut()
            .find(|submission| submission.player_id == player_id)
        {
            existing.answer = answer;
        } else {
            self.submissions.push(AnswerSubmission {
                player_id,
                player_name: player.display_name.clone(),
                answer,
            });
        }

        self.updated_at = SystemTime::now();
        Ok(())
    }
}
