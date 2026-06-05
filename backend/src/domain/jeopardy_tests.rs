use super::jeopardy::{
    Category, Clue, GameAction, GameError, GamePhase, GameScenario, JeopardyGame, PlayerState,
    RoundBoard,
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
        starting_selector: 1,
    }
}

#[test]
fn repeated_clue_selection_is_rejected_without_corrupting_state() {
    let mut game = JeopardyGame::new(scenario_with_two_clues()).expect("scenario should build");

    game.apply(GameAction::SelectClue {
        player_id: 1,
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
        player_id: 1,
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
        player_id: 1,
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
        player_id: 1,
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
