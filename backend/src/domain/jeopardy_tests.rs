use super::jeopardy::{
    Category, Clue, GameAction, GameError, GamePhase, GameScenario, JeopardyGame, PlayerState,
    RoundBoard, Selector,
};

fn scenario_with_two_clues() -> GameScenario {
    GameScenario {
        players: vec![
            PlayerState {
                id: 1,
                name: "Ada".to_owned(),
                score: 0,
                final_wager: None,
                final_answer_correct: None,
            },
            PlayerState {
                id: 2,
                name: "Grace".to_owned(),
                score: 0,
                final_wager: None,
                final_answer_correct: None,
            },
        ],
        rounds: vec![RoundBoard {
            name: "Jeopardy".to_owned(),
            categories: vec![Category {
                title: "Rust".to_owned(),
                clues: vec![
                    Clue {
                        id: 1,
                        label: "Ownership".to_owned(),
                        value: 200,
                        is_daily_double: false,
                        answered: false,
                    },
                    Clue {
                        id: 2,
                        label: "Borrowing".to_owned(),
                        value: 400,
                        is_daily_double: false,
                        answered: false,
                    },
                ],
            }],
        }],
        final_jeopardy: None,
    }
}

#[test]
fn repeated_clue_selection_is_rejected_without_corrupting_state() {
    let mut game = JeopardyGame::new(scenario_with_two_clues()).expect("scenario should build");

    game.apply(GameAction::SelectClue {
        actor: Selector::Moderator,
        category_index: 0,
        clue_index: 0,
    })
    .expect("first clue should be selectable");
    game.apply(GameAction::AttemptAnswer {
        player_id: 1,
        correct: true,
    })
    .expect("correct answer should resolve the clue");

    let snapshot = game.state().clone();
    let result = game.apply(GameAction::SelectClue {
        actor: Selector::Player(1),
        category_index: 0,
        clue_index: 0,
    });

    assert_eq!(result, Err(GameError::ClueAlreadyAnswered));
    assert_eq!(game.state(), &snapshot);
}

#[test]
fn single_round_game_can_finish_after_all_clues_are_resolved() {
    let mut game = JeopardyGame::new(scenario_with_two_clues()).expect("scenario should build");

    game.apply(GameAction::SelectClue {
        actor: Selector::Moderator,
        category_index: 0,
        clue_index: 0,
    })
    .expect("first clue should be selectable");
    game.apply(GameAction::AttemptAnswer {
        player_id: 1,
        correct: true,
    })
    .expect("first clue should resolve");
    game.apply(GameAction::SelectClue {
        actor: Selector::Player(1),
        category_index: 0,
        clue_index: 1,
    })
    .expect("second clue should be selectable");
    game.apply(GameAction::AttemptAnswer {
        player_id: 2,
        correct: true,
    })
    .expect("second clue should resolve");

    assert_eq!(game.state().phase, GamePhase::Completed);
    assert_eq!(game.state().players[0].score, 200);
    assert_eq!(game.state().players[1].score, 400);
}

#[test]
fn game_opens_with_the_moderator_in_control() {
    let game = JeopardyGame::new(scenario_with_two_clues()).expect("scenario should build");
    assert_eq!(game.state().current_selector, Selector::Moderator);
}

#[test]
fn marking_a_player_correct_transfers_control_to_that_player() {
    let mut game = JeopardyGame::new(scenario_with_two_clues()).expect("scenario should build");

    game.apply(GameAction::SelectClue {
        actor: Selector::Moderator,
        category_index: 0,
        clue_index: 0,
    })
    .expect("moderator should pick the first clue");
    game.apply(GameAction::AttemptAnswer {
        player_id: 2,
        correct: true,
    })
    .expect("correct answer should resolve the clue");

    assert_eq!(game.state().current_selector, Selector::Player(2));
}

#[test]
fn a_player_cannot_select_when_it_is_not_their_turn() {
    let mut game = JeopardyGame::new(scenario_with_two_clues()).expect("scenario should build");

    game.apply(GameAction::SelectClue {
        actor: Selector::Moderator,
        category_index: 0,
        clue_index: 0,
    })
    .expect("moderator should pick the first clue");
    game.apply(GameAction::AttemptAnswer {
        player_id: 1,
        correct: true,
    })
    .expect("player one should win control");

    let snapshot = game.state().clone();
    let result = game.apply(GameAction::SelectClue {
        actor: Selector::Player(2),
        category_index: 0,
        clue_index: 1,
    });

    assert_eq!(result, Err(GameError::NotCurrentSelector));
    assert_eq!(game.state(), &snapshot);
}

#[test]
fn the_moderator_cannot_select_during_a_players_turn() {
    let mut game = JeopardyGame::new(scenario_with_two_clues()).expect("scenario should build");

    game.apply(GameAction::SelectClue {
        actor: Selector::Moderator,
        category_index: 0,
        clue_index: 0,
    })
    .expect("moderator should pick the first clue");
    game.apply(GameAction::AttemptAnswer {
        player_id: 1,
        correct: true,
    })
    .expect("player one should win control");

    let snapshot = game.state().clone();
    let result = game.apply(GameAction::SelectClue {
        actor: Selector::Moderator,
        category_index: 0,
        clue_index: 1,
    });

    assert_eq!(result, Err(GameError::NotCurrentSelector));
    assert_eq!(game.state(), &snapshot);
}

#[test]
fn control_stays_with_the_moderator_when_nobody_answers_the_first_clue() {
    let mut game = JeopardyGame::new(scenario_with_two_clues()).expect("scenario should build");

    game.apply(GameAction::SelectClue {
        actor: Selector::Moderator,
        category_index: 0,
        clue_index: 0,
    })
    .expect("moderator should pick the first clue");
    game.apply(GameAction::AttemptAnswer {
        player_id: 1,
        correct: false,
    })
    .expect("first wrong answer");
    game.apply(GameAction::AttemptAnswer {
        player_id: 2,
        correct: false,
    })
    .expect("second wrong answer closes the clue");

    assert_eq!(game.state().phase, GamePhase::RoundSelection);
    assert_eq!(game.state().current_selector, Selector::Moderator);
}

#[test]
fn control_stays_with_the_previous_player_when_nobody_answers() {
    let mut game = JeopardyGame::new(scenario_with_two_clues()).expect("scenario should build");

    game.apply(GameAction::SelectClue {
        actor: Selector::Moderator,
        category_index: 0,
        clue_index: 0,
    })
    .expect("moderator should pick the first clue");
    game.apply(GameAction::AttemptAnswer {
        player_id: 1,
        correct: true,
    })
    .expect("player one should win control");

    game.apply(GameAction::SelectClue {
        actor: Selector::Player(1),
        category_index: 0,
        clue_index: 1,
    })
    .expect("player one should pick the next clue");
    game.apply(GameAction::AttemptAnswer {
        player_id: 1,
        correct: false,
    })
    .expect("first wrong answer");
    game.apply(GameAction::AttemptAnswer {
        player_id: 2,
        correct: false,
    })
    .expect("second wrong answer closes the clue");

    // Both clues are now resolved, so the game is over but control never moved.
    assert_eq!(game.state().phase, GamePhase::Completed);
    assert_eq!(game.state().current_selector, Selector::Player(1));
}
