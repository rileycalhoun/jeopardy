use crate::{
    content::models::QuestionPack,
    domain::jeopardy::{Category, Clue, FinalJeopardyClue, GameScenario, PlayerState, RoundBoard},
    players::models::PlayerSummary,
};

pub fn build_scenario(
    pack: &QuestionPack,
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
    let rounds = pack
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

    let final_jeopardy = pack
        .final_jeopardy
        .as_ref()
        .map(|final_clue| FinalJeopardyClue {
            category: final_clue.category.clone(),
            clue: final_clue.question.clone(),
        });

    Ok(GameScenario {
        starting_selector: scenario_players[0].id,
        players: scenario_players,
        rounds,
        final_jeopardy,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::models::{PackCategory, PackClue, PackRound};

    #[test]
    fn builds_engine_scenarios_from_packs() {
        let pack = QuestionPack {
            id: "classic".to_owned(),
            title: "Classic".to_owned(),
            rounds: vec![PackRound {
                name: "Jeopardy".to_owned(),
                categories: vec![PackCategory {
                    title: "Rust".to_owned(),
                    clues: vec![PackClue {
                        label: "$200".to_owned(),
                        question: "This keyword creates an immutable binding.".to_owned(),
                        answer: "What is let?".to_owned(),
                        value: 200,
                        daily_double: true,
                    }],
                }],
            }],
            final_jeopardy: None,
        };
        let players = vec![PlayerSummary {
            id: 42,
            display_name: "Ada".to_owned(),
        }];

        let scenario = build_scenario(&pack, &players).expect("scenario should build");

        assert_eq!(scenario.starting_selector, 42);
        assert_eq!(scenario.players[0].name, "Ada");
        assert_eq!(scenario.rounds[0].categories[0].title, "Rust");
        assert_eq!(scenario.rounds[0].categories[0].clues[0].value, 200);
        assert!(scenario.rounds[0].categories[0].clues[0].is_daily_double);
    }

    #[test]
    fn rejects_scenarios_without_players() {
        let pack = QuestionPack {
            id: "classic".to_owned(),
            title: "Classic".to_owned(),
            rounds: Vec::new(),
            final_jeopardy: None,
        };

        let err = build_scenario(&pack, &[]).expect_err("players are required");

        assert_eq!(err, "cannot start a game without players");
    }
}
