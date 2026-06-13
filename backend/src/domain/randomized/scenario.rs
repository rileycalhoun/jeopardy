use rand::{RngExt, prelude::IndexedRandom, rngs::StdRng, seq::SliceRandom};

use crate::domain::jeopardy::{
    Category, Clue, FinalJeopardyClue, GameScenario, PlayerState, RoundBoard,
};

const NAME_PARTS: &[&str] = &[
    "Ada",
    "Grace",
    "Linus",
    "Margaret",
    "Ken",
    "Alex",
    "Zoë",
    "Renée",
    "Player-One",
    "Buzz++",
    "Δelta",
    "Team Rocket",
    "Quizmaster",
];

const CATEGORY_WORDS: &[&str] = &[
    "Science", "History", "Rust", "Math", "Music", "Rivers", "Space", "Cities", "Animals", "Food",
];

pub fn generate_scenario(rng: &mut StdRng) -> GameScenario {
    let player_count = rng.random_range(1..=4);
    let players = (0..player_count)
        .map(|index| PlayerState {
            id: (index + 1) as u32,
            name: random_name(rng, index),
            score: 0,
            final_wager: None,
            final_answer_correct: None,
        })
        .collect::<Vec<_>>();

    let round_count = rng.random_range(1..=2);
    let rounds = (0..round_count)
        .map(|round_index| generate_round(rng, round_index))
        .collect::<Vec<_>>();

    let final_jeopardy = if rng.random_bool(0.85) {
        Some(FinalJeopardyClue {
            category: random_category_title(rng),
            clue: format!("Final clue about {}", random_category_title(rng)),
        })
    } else {
        None
    };

    GameScenario {
        players,
        rounds,
        final_jeopardy,
    }
}

pub fn tie_breaker_scenario() -> GameScenario {
    GameScenario {
        players: vec![
            PlayerState {
                id: 1,
                name: "Tie One".to_owned(),
                score: 0,
                final_wager: None,
                final_answer_correct: None,
            },
            PlayerState {
                id: 2,
                name: "Tie Two".to_owned(),
                score: 0,
                final_wager: None,
                final_answer_correct: None,
            },
        ],
        rounds: vec![RoundBoard {
            name: "Single".to_owned(),
            categories: vec![Category {
                title: "Final Setup".to_owned(),
                clues: vec![Clue {
                    id: 1,
                    label: "Warmup".to_owned(),
                    value: 200,
                    is_daily_double: false,
                    answered: false,
                }],
            }],
        }],
        final_jeopardy: Some(FinalJeopardyClue {
            category: "Tie".to_owned(),
            clue: "Everybody ties".to_owned(),
        }),
    }
}

pub fn all_zero_scores_scenario() -> GameScenario {
    GameScenario {
        players: vec![
            PlayerState {
                id: 1,
                name: "Zero".to_owned(),
                score: 0,
                final_wager: None,
                final_answer_correct: None,
            },
            PlayerState {
                id: 2,
                name: "Also Zero".to_owned(),
                score: 0,
                final_wager: None,
                final_answer_correct: None,
            },
        ],
        rounds: vec![RoundBoard {
            name: "Empty".to_owned(),
            categories: vec![Category {
                title: "No Clues".to_owned(),
                clues: vec![],
            }],
        }],
        final_jeopardy: Some(FinalJeopardyClue {
            category: "Zero".to_owned(),
            clue: "Zero wager edge".to_owned(),
        }),
    }
}

pub fn all_negative_scores_scenario() -> GameScenario {
    GameScenario {
        players: vec![
            PlayerState {
                id: 1,
                name: "Negative One".to_owned(),
                score: -400,
                final_wager: None,
                final_answer_correct: None,
            },
            PlayerState {
                id: 2,
                name: "Negative Two".to_owned(),
                score: -200,
                final_wager: None,
                final_answer_correct: None,
            },
        ],
        rounds: vec![RoundBoard {
            name: "Spent".to_owned(),
            categories: vec![Category {
                title: "Aftermath".to_owned(),
                clues: vec![],
            }],
        }],
        final_jeopardy: Some(FinalJeopardyClue {
            category: "No Final".to_owned(),
            clue: "Nobody should qualify".to_owned(),
        }),
    }
}

pub fn daily_double_max_wager_scenario() -> GameScenario {
    GameScenario {
        players: vec![PlayerState {
            id: 1,
            name: "Max Wager".to_owned(),
            score: 0,
            final_wager: None,
            final_answer_correct: None,
        }],
        rounds: vec![RoundBoard {
            name: "Jeopardy".to_owned(),
            categories: vec![Category {
                title: "Risk".to_owned(),
                clues: vec![Clue {
                    id: 1,
                    label: "Double down".to_owned(),
                    value: 1000,
                    is_daily_double: true,
                    answered: false,
                }],
            }],
        }],
        final_jeopardy: None,
    }
}

pub fn completed_board_scenario() -> GameScenario {
    GameScenario {
        players: vec![PlayerState {
            id: 1,
            name: "Done".to_owned(),
            score: 0,
            final_wager: None,
            final_answer_correct: None,
        }],
        rounds: vec![RoundBoard {
            name: "Finished".to_owned(),
            categories: vec![Category {
                title: "Empty".to_owned(),
                clues: vec![],
            }],
        }],
        final_jeopardy: None,
    }
}

fn generate_round(rng: &mut StdRng, round_index: usize) -> RoundBoard {
    let category_count = rng.random_range(2..=6);
    let clues_per_category = rng.random_range(1..=5);
    let base_value = if round_index == 0 { 200 } else { 400 };
    let daily_double_targets = if round_index == 0 { 1 } else { 2 };
    let total_clues = category_count * clues_per_category;
    let mut daily_double_indexes = (0..total_clues).collect::<Vec<_>>();
    daily_double_indexes.shuffle(rng);
    daily_double_indexes.truncate(daily_double_targets.min(total_clues));

    let mut clue_counter = 1u32;
    let categories = (0..category_count)
        .map(|category_index| {
            let clues = (0..clues_per_category)
                .map(|clue_index| {
                    let flattened_index = category_index * clues_per_category + clue_index;
                    let clue = Clue {
                        id: clue_counter,
                        label: format!(
                            "{} for {}",
                            random_category_title(rng),
                            base_value * (clue_index as i32 + 1)
                        ),
                        value: base_value * (clue_index as i32 + 1),
                        is_daily_double: daily_double_indexes.contains(&flattened_index),
                        answered: false,
                    };
                    clue_counter += 1;
                    clue
                })
                .collect::<Vec<_>>();

            Category {
                title: random_category_title(rng),
                clues,
            }
        })
        .collect::<Vec<_>>();

    RoundBoard {
        name: if round_index == 0 {
            "Jeopardy".to_owned()
        } else {
            "Double Jeopardy".to_owned()
        },
        categories,
    }
}

fn random_name(rng: &mut StdRng, index: usize) -> String {
    let mut name = NAME_PARTS
        .choose(rng)
        .copied()
        .unwrap_or("Player")
        .to_owned();

    if rng.random_bool(0.25) {
        name.push_str(" !!!");
    }

    if rng.random_bool(0.15) {
        name = format!("{name} {}", "X".repeat(20 + index));
    }

    if rng.random_bool(0.20) {
        name.push_str(" & Co.");
    }

    name
}

fn random_category_title(rng: &mut StdRng) -> String {
    CATEGORY_WORDS
        .choose(rng)
        .copied()
        .unwrap_or("Trivia")
        .to_owned()
}
