use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub type PlayerId = u32;

/// Whoever is allowed to pick the next clue. Games open with the moderator in
/// control; after a contestant is marked correct, control moves to that player.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Selector {
    /// The host/moderator. Always allowed to select (first clue + override).
    Moderator,
    /// A specific contestant who earned the right to pick the next clue.
    Player(PlayerId),
}

impl Selector {
    /// The contestant id when a player holds control, or `None` for the moderator.
    pub fn player_id(self) -> Option<PlayerId> {
        match self {
            Selector::Moderator => None,
            Selector::Player(id) => Some(id),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: PlayerId,
    pub name: String,
    pub score: i32,
    pub final_wager: Option<i32>,
    pub final_answer_correct: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clue {
    pub id: u32,
    pub label: String,
    pub value: i32,
    pub is_daily_double: bool,
    pub answered: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Category {
    pub title: String,
    pub clues: Vec<Clue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundBoard {
    pub name: String,
    pub categories: Vec<Category>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalJeopardyClue {
    pub category: String,
    pub clue: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameScenario {
    pub players: Vec<PlayerState>,
    pub rounds: Vec<RoundBoard>,
    pub final_jeopardy: Option<FinalJeopardyClue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    RoundSelection,
    ClueOpen,
    DailyDoubleWager,
    DailyDoubleAnswer,
    FinalJeopardyWager,
    FinalJeopardyReveal,
    Completed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveClue {
    pub round_index: usize,
    pub category_index: usize,
    pub clue_index: usize,
    pub selector_before_clue: Selector,
    pub attempted_player_ids: Vec<PlayerId>,
    pub daily_double_wager: Option<i32>,
    /// The contestant resolving a daily double. Equals the selector when a
    /// player is in control, or whoever the moderator designated for a
    /// moderator-selected daily double (e.g. the very first clue).
    pub daily_double_player: Option<PlayerId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    pub phase: GamePhase,
    pub current_round: usize,
    pub players: Vec<PlayerState>,
    pub rounds: Vec<RoundBoard>,
    pub current_selector: Selector,
    pub active_clue: Option<ActiveClue>,
    pub final_jeopardy: Option<FinalJeopardyClue>,
    pub final_reveal_order: Vec<PlayerId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameAction {
    SelectClue {
        actor: Selector,
        category_index: usize,
        clue_index: usize,
    },
    SkipClue,
    AttemptAnswer {
        player_id: PlayerId,
        correct: bool,
    },
    SubmitDailyDoubleWager {
        player_id: PlayerId,
        amount: i32,
    },
    ResolveDailyDouble {
        player_id: PlayerId,
        correct: bool,
    },
    SubmitFinalWager {
        player_id: PlayerId,
        amount: i32,
    },
    ResolveFinalAnswer {
        player_id: PlayerId,
        correct: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameError {
    WrongPhase,
    InvalidPlayer,
    NotCurrentSelector,
    ClueAlreadyAnswered,
    ClueOutOfBounds,
    AlreadyAttempted,
    NoActiveClue,
    InvalidWager { min: i32, max: i32, attempted: i32 },
    FinalWagerAlreadySubmitted,
    FinalAnswerAlreadyResolved,
    OutOfRevealOrder,
    InvalidScenario(&'static str),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JeopardyGame {
    state: GameState,
}

impl JeopardyGame {
    pub fn new(scenario: GameScenario) -> Result<Self, GameError> {
        if scenario.players.is_empty() {
            return Err(GameError::InvalidScenario(
                "scenario must include at least one player",
            ));
        }

        if scenario.rounds.is_empty() && scenario.final_jeopardy.is_none() {
            return Err(GameError::InvalidScenario(
                "scenario must include at least one round or final clue",
            ));
        }

        let player_ids = scenario
            .players
            .iter()
            .map(|player| player.id)
            .collect::<Vec<_>>();
        let unique_ids = player_ids.iter().copied().collect::<HashSet<_>>();
        if player_ids.len() != unique_ids.len() {
            return Err(GameError::InvalidScenario("player ids must be unique"));
        }

        let mut state = GameState {
            phase: GamePhase::RoundSelection,
            current_round: 0,
            players: scenario.players,
            rounds: scenario.rounds,
            // Games open with the moderator choosing the first clue; control
            // passes to a contestant once one is marked correct.
            current_selector: Selector::Moderator,
            active_clue: None,
            final_jeopardy: scenario.final_jeopardy,
            final_reveal_order: Vec::new(),
        };

        for player in &mut state.players {
            player.final_wager = None;
            player.final_answer_correct = None;
        }

        let mut game = Self { state };
        game.sync_terminal_phase_if_needed();
        Ok(game)
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn apply(&mut self, _action: GameAction) -> Result<(), GameError> {
        match _action {
            GameAction::SelectClue {
                actor,
                category_index,
                clue_index,
            } => self.select_clue(actor, category_index, clue_index),
            GameAction::SkipClue => self.skip_clue(),
            GameAction::AttemptAnswer { player_id, correct } => {
                self.attempt_answer(player_id, correct)
            }
            GameAction::SubmitDailyDoubleWager { player_id, amount } => {
                self.submit_daily_double_wager(player_id, amount)
            }
            GameAction::ResolveDailyDouble { player_id, correct } => {
                self.resolve_daily_double(player_id, correct)
            }
            GameAction::SubmitFinalWager { player_id, amount } => {
                self.submit_final_wager(player_id, amount)
            }
            GameAction::ResolveFinalAnswer { player_id, correct } => {
                self.resolve_final_answer(player_id, correct)
            }
        }
    }
}

impl JeopardyGame {
    fn select_clue(
        &mut self,
        actor: Selector,
        category_index: usize,
        clue_index: usize,
    ) -> Result<(), GameError> {
        if self.state.phase != GamePhase::RoundSelection {
            return Err(GameError::WrongPhase);
        }

        self.ensure_can_select(actor)?;
        let is_daily_double = self
            .current_round_board()
            .categories
            .get(category_index)
            .and_then(|category| category.clues.get(clue_index))
            .ok_or(GameError::ClueOutOfBounds)?
            .is_daily_double;

        if self
            .current_round_board()
            .categories
            .get(category_index)
            .and_then(|category| category.clues.get(clue_index))
            .ok_or(GameError::ClueOutOfBounds)?
            .answered
        {
            return Err(GameError::ClueAlreadyAnswered);
        }

        self.state.active_clue = Some(ActiveClue {
            round_index: self.state.current_round,
            category_index,
            clue_index,
            selector_before_clue: self.state.current_selector,
            attempted_player_ids: Vec::new(),
            daily_double_wager: None,
            daily_double_player: None,
        });
        self.state.phase = if is_daily_double {
            GamePhase::DailyDoubleWager
        } else {
            GamePhase::ClueOpen
        };
        Ok(())
    }

    fn attempt_answer(&mut self, player_id: PlayerId, correct: bool) -> Result<(), GameError> {
        if self.state.phase != GamePhase::ClueOpen {
            return Err(GameError::WrongPhase);
        }

        let player_index = self.player_index(player_id)?;
        let clue = self.active_clue()?.clone();
        if clue.attempted_player_ids.contains(&player_id) {
            return Err(GameError::AlreadyAttempted);
        }

        let value = self.active_clue_value()?;
        if correct {
            self.state.players[player_index].score += value;
            self.state.current_selector = Selector::Player(player_id);
            self.mark_active_clue_answered()?;
            self.state.active_clue = None;
            self.advance_after_clue();
            return Ok(());
        }

        self.state.players[player_index].score -= value;
        if let Some(active) = &mut self.state.active_clue {
            active.attempted_player_ids.push(player_id);
        }

        let everyone_attempted = self.state.players.iter().all(|player| {
            self.active_clue()
                .is_ok_and(|active| active.attempted_player_ids.contains(&player.id))
        });
        if everyone_attempted {
            self.mark_active_clue_answered()?;
            self.state.active_clue = None;
            self.advance_after_clue();
        }

        Ok(())
    }

    fn skip_clue(&mut self) -> Result<(), GameError> {
        if self.state.phase != GamePhase::ClueOpen {
            return Err(GameError::WrongPhase);
        }

        self.mark_active_clue_answered()?;
        self.state.active_clue = None;
        self.advance_after_clue();
        Ok(())
    }

    fn submit_daily_double_wager(
        &mut self,
        player_id: PlayerId,
        amount: i32,
    ) -> Result<(), GameError> {
        if self.state.phase != GamePhase::DailyDoubleWager {
            return Err(GameError::WrongPhase);
        }

        self.ensure_daily_double_player(player_id)?;
        let max = self.max_daily_double_wager(player_id)?;
        if !(1..=max).contains(&amount) {
            return Err(GameError::InvalidWager {
                min: 1,
                max,
                attempted: amount,
            });
        }

        if let Some(active) = &mut self.state.active_clue {
            active.daily_double_wager = Some(amount);
            active.daily_double_player = Some(player_id);
        }
        self.state.phase = GamePhase::DailyDoubleAnswer;
        Ok(())
    }

    fn resolve_daily_double(
        &mut self,
        player_id: PlayerId,
        correct: bool,
    ) -> Result<(), GameError> {
        if self.state.phase != GamePhase::DailyDoubleAnswer {
            return Err(GameError::WrongPhase);
        }

        // The contestant resolving must be the one who placed the wager.
        if self.active_clue()?.daily_double_player != Some(player_id) {
            return Err(GameError::NotCurrentSelector);
        }
        let player_index = self.player_index(player_id)?;
        let wager = self
            .active_clue()?
            .daily_double_wager
            .ok_or(GameError::InvalidScenario(
                "daily double wager must be set before resolution",
            ))?;

        if correct {
            self.state.players[player_index].score += wager;
        } else {
            self.state.players[player_index].score -= wager;
        }

        self.mark_active_clue_answered()?;
        self.state.active_clue = None;
        self.advance_after_clue();
        Ok(())
    }

    fn submit_final_wager(&mut self, player_id: PlayerId, amount: i32) -> Result<(), GameError> {
        if self.state.phase != GamePhase::FinalJeopardyWager {
            return Err(GameError::WrongPhase);
        }

        let player_index = self.player_index(player_id)?;
        if !self.is_final_eligible(player_id)? {
            return Err(GameError::InvalidPlayer);
        }
        if self.state.players[player_index].final_wager.is_some() {
            return Err(GameError::FinalWagerAlreadySubmitted);
        }

        let max = self.state.players[player_index].score.max(0);
        if !(0..=max).contains(&amount) {
            return Err(GameError::InvalidWager {
                min: 0,
                max,
                attempted: amount,
            });
        }

        self.state.players[player_index].final_wager = Some(amount);
        if self
            .eligible_final_player_ids()
            .into_iter()
            .all(|eligible_id| {
                self.player(eligible_id)
                    .is_ok_and(|player| player.final_wager.is_some())
            })
        {
            self.state.phase = GamePhase::FinalJeopardyReveal;
            let mut reveal = self.eligible_final_player_ids();
            reveal.sort_by_key(|player_id| {
                self.player(*player_id)
                    .map(|player| player.score)
                    .unwrap_or_default()
            });
            self.state.final_reveal_order = reveal;
        }

        Ok(())
    }

    fn resolve_final_answer(
        &mut self,
        player_id: PlayerId,
        correct: bool,
    ) -> Result<(), GameError> {
        if self.state.phase != GamePhase::FinalJeopardyReveal {
            return Err(GameError::WrongPhase);
        }

        let next_player = self
            .state
            .final_reveal_order
            .iter()
            .find(|candidate| {
                self.player(**candidate)
                    .is_ok_and(|player| player.final_answer_correct.is_none())
            })
            .copied()
            .ok_or(GameError::WrongPhase)?;
        if next_player != player_id {
            return Err(GameError::OutOfRevealOrder);
        }

        let player_index = self.player_index(player_id)?;
        if self.state.players[player_index]
            .final_answer_correct
            .is_some()
        {
            return Err(GameError::FinalAnswerAlreadyResolved);
        }

        let wager =
            self.state.players[player_index]
                .final_wager
                .ok_or(GameError::InvalidScenario(
                    "final wager must be submitted before reveal",
                ))?;
        if correct {
            self.state.players[player_index].score += wager;
        } else {
            self.state.players[player_index].score -= wager;
        }
        self.state.players[player_index].final_answer_correct = Some(correct);

        if self.state.final_reveal_order.iter().all(|candidate| {
            self.player(*candidate)
                .is_ok_and(|player| player.final_answer_correct.is_some())
        }) {
            self.state.phase = GamePhase::Completed;
        }

        Ok(())
    }

    fn sync_terminal_phase_if_needed(&mut self) {
        if self.state.rounds.is_empty() || self.current_round_complete() {
            self.advance_after_clue();
        }
    }

    fn advance_after_clue(&mut self) {
        if !self.current_round_complete() {
            self.state.phase = GamePhase::RoundSelection;
            return;
        }

        if self.state.current_round + 1 < self.state.rounds.len() {
            self.state.current_round += 1;
            self.state.phase = GamePhase::RoundSelection;
            if self.current_round_complete() {
                self.advance_after_clue();
            }
            return;
        }

        if self.state.final_jeopardy.is_some() && !self.eligible_final_player_ids().is_empty() {
            self.state.phase = GamePhase::FinalJeopardyWager;
            return;
        }

        self.state.phase = GamePhase::Completed;
    }

    fn current_round_complete(&self) -> bool {
        self.current_round_board()
            .categories
            .iter()
            .flat_map(|category| &category.clues)
            .all(|clue| clue.answered)
    }

    fn current_round_board(&self) -> &RoundBoard {
        &self.state.rounds[self.state.current_round]
    }

    fn active_clue(&self) -> Result<&ActiveClue, GameError> {
        self.state
            .active_clue
            .as_ref()
            .ok_or(GameError::NoActiveClue)
    }

    fn active_clue_value(&self) -> Result<i32, GameError> {
        let active = self.active_clue()?;
        self.state.rounds[active.round_index]
            .categories
            .get(active.category_index)
            .and_then(|category| category.clues.get(active.clue_index))
            .map(|clue| clue.value)
            .ok_or(GameError::ClueOutOfBounds)
    }

    fn mark_active_clue_answered(&mut self) -> Result<(), GameError> {
        let active = self.active_clue()?.clone();
        let clue = self
            .state
            .rounds
            .get_mut(active.round_index)
            .and_then(|round| round.categories.get_mut(active.category_index))
            .and_then(|category| category.clues.get_mut(active.clue_index))
            .ok_or(GameError::ClueOutOfBounds)?;
        clue.answered = true;
        Ok(())
    }

    /// Whether `actor` may pick the next clue: only whoever currently holds
    /// control. The moderator picks while in control (the first clue, or after a
    /// clue nobody answered correctly), and a contestant picks once they have
    /// earned control. The moderator cannot pick over a contestant's turn.
    fn ensure_can_select(&self, actor: Selector) -> Result<(), GameError> {
        if let Selector::Player(player_id) = actor {
            self.player_index(player_id)?;
        }
        if self.state.current_selector == actor {
            Ok(())
        } else {
            Err(GameError::NotCurrentSelector)
        }
    }

    /// Whether `player_id` may wager/resolve the active daily double. When a
    /// contestant holds control it must be that contestant; when the moderator
    /// holds control (e.g. a daily double as the first clue) the moderator may
    /// designate any valid contestant.
    fn ensure_daily_double_player(&self, player_id: PlayerId) -> Result<(), GameError> {
        self.player_index(player_id)?;
        match self.state.current_selector {
            Selector::Moderator => Ok(()),
            Selector::Player(selector_id) => {
                if selector_id == player_id {
                    Ok(())
                } else {
                    Err(GameError::NotCurrentSelector)
                }
            }
        }
    }

    fn player_index(&self, player_id: PlayerId) -> Result<usize, GameError> {
        self.state
            .players
            .iter()
            .position(|player| player.id == player_id)
            .ok_or(GameError::InvalidPlayer)
    }

    fn player(&self, player_id: PlayerId) -> Result<&PlayerState, GameError> {
        let index = self.player_index(player_id)?;
        Ok(&self.state.players[index])
    }

    fn max_daily_double_wager(&self, player_id: PlayerId) -> Result<i32, GameError> {
        let player = self.player(player_id)?;
        let round_max = self
            .current_round_board()
            .categories
            .iter()
            .flat_map(|category| &category.clues)
            .map(|clue| clue.value)
            .max()
            .unwrap_or(1);
        Ok(round_max.max(player.score.max(0)).max(1))
    }

    fn eligible_final_player_ids(&self) -> Vec<PlayerId> {
        self.state
            .players
            .iter()
            .filter(|player| player.score >= 0)
            .map(|player| player.id)
            .collect()
    }

    fn is_final_eligible(&self, player_id: PlayerId) -> Result<bool, GameError> {
        Ok(self.player(player_id)?.score >= 0)
    }
}
