use rand::{RngExt, SeedableRng, prelude::IndexedRandom, rngs::StdRng};

use crate::domain::jeopardy::{
    GameAction, GameError, GamePhase, GameState, JeopardyGame, PlayerId,
};

use super::{invariants::assert_invariants, scenario::generate_scenario};

#[derive(Debug)]
struct PlannedAction {
    action: GameAction,
    intentionally_invalid: bool,
}

pub fn run_randomized_case(
    root_seed: u64,
    case_index: usize,
    case_seed: u64,
    max_actions: usize,
) -> Result<(), String> {
    let mut rng = StdRng::seed_from_u64(case_seed);
    let scenario = generate_scenario(&mut rng);
    let mut game = JeopardyGame::new(scenario.clone())
        .map_err(|error| format!("failed to build scenario for seed {case_seed}: {error:?}"))?;
    let mut history = Vec::new();

    assert_invariants(game.state(), None).map_err(|error| {
        format_failure(
            root_seed,
            case_index,
            case_seed,
            0,
            &history,
            None,
            &error,
            game.state(),
        )
    })?;

    for action_index in 0..max_actions {
        if game.state().phase == GamePhase::Completed {
            return Ok(());
        }

        let previous = game.state().clone();
        let planned = next_action(game.state(), &mut rng).map_err(|error| {
            format_failure(
                root_seed,
                case_index,
                case_seed,
                action_index,
                &history,
                None,
                &error,
                game.state(),
            )
        })?;
        let action_label = format!("{:?}", planned.action);
        let result = game.apply(planned.action.clone());

        match result {
            Ok(()) => history.push(action_label),
            Err(error) if planned.intentionally_invalid && is_expected_validation_error(&error) => {
                history.push(format!("{action_label} => expected {error:?}"));
            }
            Err(error) => {
                return Err(format_failure(
                    root_seed,
                    case_index,
                    case_seed,
                    action_index,
                    &history,
                    Some(&planned.action),
                    &format!("unexpected engine error: {error:?}"),
                    game.state(),
                ));
            }
        }

        if let Err(error) = assert_invariants(game.state(), Some(&previous)) {
            return Err(format_failure(
                root_seed,
                case_index,
                case_seed,
                action_index,
                &history,
                Some(&planned.action),
                &error,
                game.state(),
            ));
        }
    }

    if game.state().phase != GamePhase::Completed {
        return Err(format_failure(
            root_seed,
            case_index,
            case_seed,
            max_actions,
            &history,
            None,
            "game did not reach a completed state before RANDOM_TEST_MAX_ACTIONS",
            game.state(),
        ));
    }

    Ok(())
}

fn next_action(state: &GameState, rng: &mut StdRng) -> Result<PlannedAction, String> {
    match state.phase {
        GamePhase::RoundSelection => next_selection_action(state, rng),
        GamePhase::ClueOpen => next_clue_action(state, rng),
        GamePhase::DailyDoubleWager => next_daily_double_wager_action(state, rng),
        GamePhase::DailyDoubleAnswer => next_daily_double_answer_action(state, rng),
        GamePhase::FinalJeopardyWager => next_final_wager_action(state, rng),
        GamePhase::FinalJeopardyReveal => next_final_reveal_action(state, rng),
        GamePhase::Completed => {
            Err("no further actions should be generated after completion".to_owned())
        }
    }
}

fn next_selection_action(state: &GameState, rng: &mut StdRng) -> Result<PlannedAction, String> {
    let selectable = unanswered_clues(state);
    if selectable.is_empty() {
        return Err("selection phase has no unanswered clues".to_owned());
    }

    let invalid_choice = rng.random_bool(0.22);
    let (category_index, clue_index) = selectable
        .choose(rng)
        .copied()
        .ok_or_else(|| "no clue available for selection".to_owned())?;

    if invalid_choice {
        let wrong_player =
            pick_wrong_player(state, state.current_selector).unwrap_or(state.current_selector);
        if rng.random_bool(0.5) {
            let answered = answered_clues(state);
            if let Some((answered_category, answered_clue)) = answered.choose(rng).copied() {
                return Ok(PlannedAction {
                    action: GameAction::SelectClue {
                        player_id: state.current_selector,
                        category_index: answered_category,
                        clue_index: answered_clue,
                    },
                    intentionally_invalid: true,
                });
            }
        }

        return Ok(PlannedAction {
            action: GameAction::SelectClue {
                player_id: wrong_player,
                category_index,
                clue_index,
            },
            intentionally_invalid: wrong_player != state.current_selector,
        });
    }

    Ok(PlannedAction {
        action: GameAction::SelectClue {
            player_id: state.current_selector,
            category_index,
            clue_index,
        },
        intentionally_invalid: false,
    })
}

fn next_clue_action(state: &GameState, rng: &mut StdRng) -> Result<PlannedAction, String> {
    let active = state
        .active_clue
        .as_ref()
        .ok_or_else(|| "clue phase is missing an active clue".to_owned())?;
    let attempted = active
        .attempted_player_ids
        .iter()
        .copied()
        .collect::<Vec<_>>();
    let available_players = state
        .players
        .iter()
        .map(|player| player.id)
        .filter(|player_id| !attempted.contains(player_id))
        .collect::<Vec<_>>();

    if available_players.is_empty() {
        return Err("clue phase left no players available to answer".to_owned());
    }

    let intentionally_invalid = rng.random_bool(0.20);
    if intentionally_invalid {
        let duplicate_player = attempted
            .first()
            .copied()
            .unwrap_or_else(|| state.players[0].id);
        return Ok(PlannedAction {
            action: GameAction::AttemptAnswer {
                player_id: duplicate_player,
                correct: rng.random_bool(0.35),
            },
            intentionally_invalid: true,
        });
    }

    let player_id = *available_players
        .choose(rng)
        .ok_or_else(|| "could not choose a player to answer".to_owned())?;
    let should_answer_correctly = if available_players.len() == 1 {
        rng.random_bool(0.5)
    } else {
        rng.random_bool(0.35)
    };
    Ok(PlannedAction {
        action: GameAction::AttemptAnswer {
            player_id,
            correct: should_answer_correctly,
        },
        intentionally_invalid: false,
    })
}

fn next_daily_double_wager_action(
    state: &GameState,
    rng: &mut StdRng,
) -> Result<PlannedAction, String> {
    let selector = state.current_selector;
    let selector_score = state
        .players
        .iter()
        .find(|player| player.id == selector)
        .map(|player| player.score.max(0))
        .ok_or_else(|| "current selector must be a valid player".to_owned())?;
    let round_max = unanswered_clue_values(state)
        .into_iter()
        .max()
        .unwrap_or(200);
    let max = selector_score.max(round_max).max(1);

    if rng.random_bool(0.25) {
        let wrong_player = pick_wrong_player(state, selector).unwrap_or(selector);
        let invalid_amount = max + rng.random_range(1..=round_max.max(1));
        return Ok(PlannedAction {
            action: GameAction::SubmitDailyDoubleWager {
                player_id: if rng.random_bool(0.5) {
                    wrong_player
                } else {
                    selector
                },
                amount: invalid_amount,
            },
            intentionally_invalid: true,
        });
    }

    Ok(PlannedAction {
        action: GameAction::SubmitDailyDoubleWager {
            player_id: selector,
            amount: rng.random_range(1..=max),
        },
        intentionally_invalid: false,
    })
}

fn next_daily_double_answer_action(
    state: &GameState,
    rng: &mut StdRng,
) -> Result<PlannedAction, String> {
    let selector = state.current_selector;
    if rng.random_bool(0.15) {
        let wrong_player = pick_wrong_player(state, selector).unwrap_or(selector);
        return Ok(PlannedAction {
            action: GameAction::ResolveDailyDouble {
                player_id: wrong_player,
                correct: rng.random_bool(0.45),
            },
            intentionally_invalid: wrong_player != selector,
        });
    }

    Ok(PlannedAction {
        action: GameAction::ResolveDailyDouble {
            player_id: selector,
            correct: rng.random_bool(0.45),
        },
        intentionally_invalid: false,
    })
}

fn next_final_wager_action(state: &GameState, rng: &mut StdRng) -> Result<PlannedAction, String> {
    let eligible = state
        .players
        .iter()
        .filter(|player| player.score >= 0 && player.final_wager.is_none())
        .map(|player| (player.id, player.score.max(0)))
        .collect::<Vec<_>>();
    let (player_id, score) = *eligible
        .choose(rng)
        .ok_or_else(|| "final jeopardy wager phase has no eligible players left".to_owned())?;

    if rng.random_bool(0.25) {
        return Ok(PlannedAction {
            action: GameAction::SubmitFinalWager {
                player_id,
                amount: score + rng.random_range(1..=200),
            },
            intentionally_invalid: true,
        });
    }

    Ok(PlannedAction {
        action: GameAction::SubmitFinalWager {
            player_id,
            amount: rng.random_range(0..=score),
        },
        intentionally_invalid: false,
    })
}

fn next_final_reveal_action(state: &GameState, rng: &mut StdRng) -> Result<PlannedAction, String> {
    let next_player = state
        .final_reveal_order
        .iter()
        .find(|player_id| {
            state
                .players
                .iter()
                .find(|player| player.id == **player_id)
                .is_some_and(|player| player.final_answer_correct.is_none())
        })
        .copied()
        .ok_or_else(|| "final reveal phase has no unresolved players".to_owned())?;

    if rng.random_bool(0.20) {
        let wrong_player = pick_wrong_player(state, next_player).unwrap_or(next_player);
        return Ok(PlannedAction {
            action: GameAction::ResolveFinalAnswer {
                player_id: wrong_player,
                correct: rng.random_bool(0.45),
            },
            intentionally_invalid: wrong_player != next_player,
        });
    }

    Ok(PlannedAction {
        action: GameAction::ResolveFinalAnswer {
            player_id: next_player,
            correct: rng.random_bool(0.45),
        },
        intentionally_invalid: false,
    })
}

fn pick_wrong_player(state: &GameState, expected: PlayerId) -> Option<PlayerId> {
    state
        .players
        .iter()
        .find(|player| player.id != expected)
        .map(|player| player.id)
}

fn unanswered_clues(state: &GameState) -> Vec<(usize, usize)> {
    state.rounds[state.current_round]
        .categories
        .iter()
        .enumerate()
        .flat_map(|(category_index, category)| {
            category
                .clues
                .iter()
                .enumerate()
                .filter(|(_, clue)| !clue.answered)
                .map(move |(clue_index, _)| (category_index, clue_index))
        })
        .collect()
}

fn answered_clues(state: &GameState) -> Vec<(usize, usize)> {
    state.rounds[state.current_round]
        .categories
        .iter()
        .enumerate()
        .flat_map(|(category_index, category)| {
            category
                .clues
                .iter()
                .enumerate()
                .filter(|(_, clue)| clue.answered)
                .map(move |(clue_index, _)| (category_index, clue_index))
        })
        .collect()
}

fn unanswered_clue_values(state: &GameState) -> Vec<i32> {
    state.rounds[state.current_round]
        .categories
        .iter()
        .flat_map(|category| &category.clues)
        .filter(|clue| !clue.answered)
        .map(|clue| clue.value)
        .collect()
}

fn is_expected_validation_error(error: &GameError) -> bool {
    matches!(
        error,
        GameError::WrongPhase
            | GameError::InvalidPlayer
            | GameError::NotCurrentSelector
            | GameError::ClueAlreadyAnswered
            | GameError::ClueOutOfBounds
            | GameError::AlreadyAttempted
            | GameError::NoActiveClue
            | GameError::InvalidWager { .. }
            | GameError::FinalWagerAlreadySubmitted
            | GameError::FinalAnswerAlreadyResolved
            | GameError::OutOfRevealOrder
    )
}

fn format_failure(
    root_seed: u64,
    case_index: usize,
    case_seed: u64,
    action_index: usize,
    history: &[String],
    failing_action: Option<&GameAction>,
    error: &str,
    state: &GameState,
) -> String {
    let snapshot = serde_json::to_string_pretty(state).unwrap_or_else(|_| format!("{state:#?}"));
    let action_history = if history.is_empty() {
        "<no previous actions>".to_owned()
    } else {
        history
            .iter()
            .enumerate()
            .map(|(index, action)| format!("  {index}: {action}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "Randomized Jeopardy test failure\n\
root seed: {root_seed}\n\
case index: {case_index}\n\
case seed: {case_seed}\n\
action index: {action_index}\n\
replay:\n\
  TEST_SEED={root_seed} RANDOM_TEST_CASES={} RANDOM_TEST_CASE_INDEX={case_index} cargo test --lib randomized_backend_engine_cases_are_deterministic_and_replayable -- --exact --nocapture\n\
action history:\n{action_history}\n\
failing action: {failing_action:?}\n\
error: {error}\n\
state snapshot:\n{snapshot}\n",
        case_index.max(1) + 1
    )
}
