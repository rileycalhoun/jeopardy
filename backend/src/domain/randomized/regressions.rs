use crate::domain::jeopardy::{GameAction, GameError, GamePhase, JeopardyGame};

use super::scenario::{
    all_negative_scores_scenario, all_zero_scores_scenario, completed_board_scenario,
    daily_double_max_wager_scenario, tie_breaker_scenario,
};
use super::{derive_case_seed, runner::run_randomized_case};

#[test]
fn regression_all_negative_scores_skip_final_and_complete_safely() {
    let game = JeopardyGame::new(all_negative_scores_scenario()).expect("scenario should build");
    assert_eq!(game.state().phase, GamePhase::Completed);
}

#[test]
fn regression_all_zero_scores_allow_zero_final_wagers() {
    let mut game = JeopardyGame::new(all_zero_scores_scenario()).expect("scenario should build");
    assert_eq!(game.state().phase, GamePhase::FinalJeopardyWager);

    game.apply(GameAction::SubmitFinalWager {
        player_id: 1,
        amount: 0,
    })
    .expect("zero score player should be able to submit a zero wager");
    game.apply(GameAction::SubmitFinalWager {
        player_id: 2,
        amount: 0,
    })
    .expect("second zero score player should be able to submit a zero wager");
    game.apply(GameAction::ResolveFinalAnswer {
        player_id: 1,
        correct: false,
    })
    .expect("first final reveal should resolve");
    game.apply(GameAction::ResolveFinalAnswer {
        player_id: 2,
        correct: true,
    })
    .expect("second final reveal should resolve");

    assert_eq!(game.state().phase, GamePhase::Completed);
}

#[test]
fn regression_tie_game_reaches_completion() {
    let mut game = JeopardyGame::new(tie_breaker_scenario()).expect("scenario should build");

    game.apply(GameAction::SelectClue {
        player_id: 1,
        category_index: 0,
        clue_index: 0,
    })
    .expect("warmup clue should be selectable");
    game.apply(GameAction::AttemptAnswer {
        player_id: 1,
        correct: true,
    })
    .expect("warmup clue should resolve");
    game.apply(GameAction::SubmitFinalWager {
        player_id: 1,
        amount: 100,
    })
    .expect("first final wager should be allowed");
    game.apply(GameAction::SubmitFinalWager {
        player_id: 2,
        amount: 0,
    })
    .expect("second final wager should be allowed");
    game.apply(GameAction::ResolveFinalAnswer {
        player_id: 2,
        correct: true,
    })
    .expect("lower score reveals first");
    game.apply(GameAction::ResolveFinalAnswer {
        player_id: 1,
        correct: false,
    })
    .expect("higher score reveals second");

    assert_eq!(game.state().phase, GamePhase::Completed);
    assert_eq!(game.state().players[0].score, 100);
    assert_eq!(game.state().players[1].score, 0);
}

#[test]
fn regression_daily_double_allows_the_maximum_wager() {
    let mut game =
        JeopardyGame::new(daily_double_max_wager_scenario()).expect("scenario should build");

    game.apply(GameAction::SelectClue {
        player_id: 1,
        category_index: 0,
        clue_index: 0,
    })
    .expect("daily double clue should be selectable");
    assert_eq!(game.state().phase, GamePhase::DailyDoubleWager);

    game.apply(GameAction::SubmitDailyDoubleWager {
        player_id: 1,
        amount: 1000,
    })
    .expect("maximum daily double wager should be valid");
    game.apply(GameAction::ResolveDailyDouble {
        player_id: 1,
        correct: true,
    })
    .expect("daily double should resolve");

    assert_eq!(game.state().players[0].score, 1000);
    assert_eq!(game.state().phase, GamePhase::Completed);
}

#[test]
fn regression_invalid_action_in_wrong_phase_is_rejected() {
    let game = JeopardyGame::new(tie_breaker_scenario()).expect("scenario should build");

    let result = {
        let mut game = game;
        game.apply(GameAction::SubmitFinalWager {
            player_id: 1,
            amount: 100,
        })
    };

    assert_eq!(result, Err(GameError::WrongPhase));
}

#[test]
fn regression_completed_board_state_starts_completed() {
    let game = JeopardyGame::new(completed_board_scenario()).expect("scenario should build");
    assert_eq!(game.state().phase, GamePhase::Completed);
}

#[test]
fn regression_saved_random_seeds_can_be_replayed() {
    let contents = include_str!("../../../tests/regressions/jeopardy_random_seeds.txt");

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split_whitespace();
        let seed = parts
            .next()
            .and_then(|value| value.parse::<u64>().ok())
            .expect("seed entry should start with a u64 seed");
        let case_index = parts
            .next()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);

        let case_seed = derive_case_seed(seed, case_index);
        run_randomized_case(seed, case_index, case_seed, 500)
            .unwrap_or_else(|report| panic!("saved regression seed failed: {report}"));
    }
}
