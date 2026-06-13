use crate::domain::jeopardy::{GamePhase, GameState, Selector};

pub fn assert_invariants(current: &GameState, previous: Option<&GameState>) -> Result<(), String> {
    assert_unique_player_ids(current)?;
    assert_valid_selector(current)?;
    assert_phase_active_clue_alignment(current)?;
    assert_answered_clues_do_not_reopen(current, previous)?;
    assert_answered_clue_count_is_monotonic(current, previous)?;
    assert_active_clue_points_to_a_live_unanswered_clue(current)?;
    assert_final_phase_is_well_formed(current)?;
    Ok(())
}

fn assert_unique_player_ids(state: &GameState) -> Result<(), String> {
    let mut ids = state
        .players
        .iter()
        .map(|player| player.id)
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    if ids.len() != state.players.len() {
        return Err("player ids must stay unique".to_owned());
    }
    Ok(())
}

fn assert_valid_selector(state: &GameState) -> Result<(), String> {
    match state.current_selector {
        // The moderator is always a valid controller (game start, overrides,
        // and clues nobody answered correctly).
        Selector::Moderator => Ok(()),
        Selector::Player(id) => {
            if state.players.iter().any(|player| player.id == id) {
                Ok(())
            } else {
                Err(format!(
                    "current selector {id} does not match any player"
                ))
            }
        }
    }
}

fn assert_phase_active_clue_alignment(state: &GameState) -> Result<(), String> {
    let clue_required = matches!(
        state.phase,
        GamePhase::ClueOpen | GamePhase::DailyDoubleWager | GamePhase::DailyDoubleAnswer
    );
    if clue_required && state.active_clue.is_none() {
        return Err(format!("phase {:?} requires an active clue", state.phase));
    }
    if !clue_required && state.active_clue.is_some() {
        return Err(format!(
            "phase {:?} should not retain an active clue",
            state.phase
        ));
    }
    Ok(())
}

fn assert_answered_clues_do_not_reopen(
    current: &GameState,
    previous: Option<&GameState>,
) -> Result<(), String> {
    let Some(previous) = previous else {
        return Ok(());
    };

    for (current_round, previous_round) in current.rounds.iter().zip(previous.rounds.iter()) {
        for (current_category, previous_category) in current_round
            .categories
            .iter()
            .zip(previous_round.categories.iter())
        {
            for (current_clue, previous_clue) in current_category
                .clues
                .iter()
                .zip(previous_category.clues.iter())
            {
                if previous_clue.answered && !current_clue.answered {
                    return Err(format!(
                        "clue {} reopened after being answered",
                        current_clue.id
                    ));
                }
            }
        }
    }

    Ok(())
}

fn assert_answered_clue_count_is_monotonic(
    current: &GameState,
    previous: Option<&GameState>,
) -> Result<(), String> {
    let Some(previous) = previous else {
        return Ok(());
    };

    let current_count = answered_clue_count(current);
    let previous_count = answered_clue_count(previous);
    if current_count < previous_count {
        return Err(format!(
            "answered clue count went backwards from {} to {}",
            previous_count, current_count
        ));
    }
    Ok(())
}

fn assert_active_clue_points_to_a_live_unanswered_clue(state: &GameState) -> Result<(), String> {
    let Some(active) = &state.active_clue else {
        return Ok(());
    };

    let clue = state
        .rounds
        .get(active.round_index)
        .and_then(|round| round.categories.get(active.category_index))
        .and_then(|category| category.clues.get(active.clue_index))
        .ok_or_else(|| "active clue points outside the board".to_owned())?;

    if clue.answered {
        return Err(format!(
            "active clue {} is already marked answered",
            clue.id
        ));
    }

    Ok(())
}

fn assert_final_phase_is_well_formed(state: &GameState) -> Result<(), String> {
    if matches!(
        state.phase,
        GamePhase::FinalJeopardyWager | GamePhase::FinalJeopardyReveal
    ) && state.final_jeopardy.is_none()
    {
        return Err("final jeopardy phase requires a final clue".to_owned());
    }

    Ok(())
}

fn answered_clue_count(state: &GameState) -> usize {
    state
        .rounds
        .iter()
        .flat_map(|round| &round.categories)
        .flat_map(|category| &category.clues)
        .filter(|clue| clue.answered)
        .count()
}
