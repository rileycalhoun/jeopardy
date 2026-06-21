use rand::{Rng, seq::IndexedRandom};

use crate::{
    content::models::{
        BoardCategory, BoardClue, BoardContent, BoardRound, CategoryPack, POINT_VALUES,
    },
    domain::jeopardy::{Category, Clue, FinalJeopardyClue, GameScenario, PlayerState, RoundBoard},
    players::models::PlayerSummary,
};

pub fn build_board_from_categories(categories: &[CategoryPack]) -> Result<BoardContent, String> {
    let mut rng = rand::rng();
    build_board_from_categories_with_rng(categories, &mut rng)
}

fn build_board_from_categories_with_rng<R: Rng + ?Sized>(
    categories: &[CategoryPack],
    rng: &mut R,
) -> Result<BoardContent, String> {
    if categories.is_empty() {
        return Err("at least one category must be selected".to_owned());
    }

    let mut board_categories = Vec::with_capacity(categories.len());
    for category in categories {
        category
            .validate()
            .map_err(|err| format!("invalid category '{}': {err}", category.id))?;

        let mut clues = Vec::with_capacity(POINT_VALUES.len());
        for points in POINT_VALUES {
            let options = category.questions_for_points(points);
            let selected = options.choose(rng).ok_or_else(|| {
                format!("category '{}' has no {points}-point questions", category.id)
            })?;
            clues.push(BoardClue {
                label: format!("${points}"),
                question: selected.question.clone(),
                answer: selected.answer.clone(),
                value: points,
                daily_double: false,
            });
        }

        board_categories.push(BoardCategory {
            id: category.id.clone(),
            title: category.title.clone(),
            clues,
        });
    }

    Ok(BoardContent {
        id: categories
            .iter()
            .map(|category| category.id.as_str())
            .collect::<Vec<_>>()
            .join("+"),
        title: "Selected Categories".to_owned(),
        rounds: vec![BoardRound {
            name: "Jeopardy".to_owned(),
            categories: board_categories,
        }],
        final_jeopardy: None,
    })
}

pub fn build_scenario(
    board: &BoardContent,
    players: &[PlayerSummary],
) -> Result<GameScenario, String> {
    if players.is_empty() {
        return Err("cannot start a game without players".to_owned());
    }

    let scenario_players = players
        .iter()
        .map(|player| {
            let id = u32::try_from(player.id)
                .map_err(|_| format!("player id '{}' is too large for the engine", player.id))?;
            Ok(PlayerState {
                id,
                name: player.display_name.clone(),
                score: 0,
                final_wager: None,
                final_answer_correct: None,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let mut next_clue_id = 1_u32;
    let rounds = board
        .rounds
        .iter()
        .map(|round| {
            let categories = round
                .categories
                .iter()
                .map(|category| {
                    let clues = category
                        .clues
                        .iter()
                        .map(|clue| {
                            let id = next_clue_id;
                            next_clue_id += 1;
                            Clue {
                                id,
                                label: clue.label.clone(),
                                value: clue.value,
                                is_daily_double: clue.daily_double,
                                answered: false,
                            }
                        })
                        .collect();

                    Category {
                        title: category.title.clone(),
                        clues,
                    }
                })
                .collect();

            RoundBoard {
                name: round.name.clone(),
                categories,
            }
        })
        .collect();

    let final_jeopardy = board
        .final_jeopardy
        .as_ref()
        .map(|final_clue| FinalJeopardyClue {
            category: final_clue.category.clone(),
            clue: final_clue.question.clone(),
        });

    Ok(GameScenario {
        players: scenario_players,
        rounds,
        final_jeopardy,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        content::models::{BoardCategory, BoardClue, BoardRound, CategoryQuestion},
        players::models::PlayerSummary,
    };

    fn board() -> BoardContent {
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
                        daily_double: true,
                    }],
                }],
            }],
            final_jeopardy: None,
        }
    }

    #[test]
    fn builds_engine_scenarios_from_generated_boards() {
        let players = vec![PlayerSummary {
            id: 42,
            display_name: "Ada".to_owned(),
        }];

        let scenario = build_scenario(&board(), &players).expect("scenario should build");

        assert_eq!(scenario.players[0].id, 42);
        assert_eq!(scenario.players[0].name, "Ada");
        assert_eq!(scenario.rounds[0].categories[0].title, "Rust");
        assert_eq!(scenario.rounds[0].categories[0].clues[0].value, 200);
        assert!(scenario.rounds[0].categories[0].clues[0].is_daily_double);
    }

    #[test]
    fn rejects_scenarios_without_players() {
        let err = build_scenario(&board(), &[]).expect_err("players are required");

        assert_eq!(err, "cannot start a game without players");
    }

    fn category_question(points: i32, suffix: &str) -> CategoryQuestion {
        CategoryQuestion {
            points,
            question: format!("{points} point question {suffix}"),
            answer: format!("{points} point answer {suffix}"),
        }
    }

    fn category_pack(id: &str, title: &str) -> CategoryPack {
        let mut questions = Vec::new();
        for points in [200, 400, 600, 800, 1000] {
            questions.push(category_question(points, "A"));
            questions.push(category_question(points, "B"));
        }

        CategoryPack {
            id: id.to_owned(),
            title: title.to_owned(),
            description: None,
            metadata: Default::default(),
            questions,
        }
    }

    #[test]
    fn generated_board_uses_one_question_per_classic_point_value_per_category() {
        let categories = vec![
            category_pack("video_games", "Video Games"),
            category_pack("movies", "Movies"),
        ];

        let board = build_board_from_categories(&categories).expect("board should build");

        assert_eq!(board.rounds.len(), 1);
        assert_eq!(board.rounds[0].categories.len(), 2);
        for category in &board.rounds[0].categories {
            let values: Vec<i32> = category.clues.iter().map(|clue| clue.value).collect();
            assert_eq!(values, vec![200, 400, 600, 800, 1000]);
            assert_eq!(category.clues.len(), 5);
        }
    }

    #[test]
    fn generated_board_does_not_include_unused_questions() {
        let categories = vec![category_pack("movies", "Movies")];

        let board = build_board_from_categories(&categories).expect("board should build");
        let generated_count = board.rounds[0].categories[0].clues.len();

        assert_eq!(generated_count, 5);
        assert!(categories[0].questions.len() > generated_count);
    }
}
