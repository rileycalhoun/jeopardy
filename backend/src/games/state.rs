use serde::Serialize;

use crate::{
    content::models::BoardContent,
    domain::jeopardy::{ActiveClue, GamePhase, GameState},
    sessions::runtime::AnswerSubmission,
};

#[derive(Clone, Debug, Serialize)]
pub struct GameView {
    pub phase: GamePhase,
    pub current_round: usize,
    /// The contestant who picks the next clue, or `None` when the moderator is
    /// in control (game start or after a clue nobody answered correctly).
    pub current_selector: Option<u32>,
    pub players: Vec<PlayerScoreView>,
    pub board: Vec<RoundView>,
    pub active_clue: Option<ActiveClueView>,
    pub final_jeopardy: Option<FinalJeopardyView>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PlayerScoreView {
    pub id: u32,
    pub name: String,
    pub score: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct RoundView {
    pub name: String,
    pub categories: Vec<CategoryView>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CategoryView {
    pub title: String,
    pub clues: Vec<ClueTileView>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ClueTileView {
    pub label: String,
    pub value: i32,
    pub answered: bool,
    pub daily_double: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct ActiveClueView {
    pub round_index: usize,
    pub category_index: usize,
    pub clue_index: usize,
    pub label: String,
    pub value: i32,
    pub question: String,
    pub answer: String,
    pub attempted_player_ids: Vec<u32>,
    pub submissions: Vec<AnswerSubmissionView>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AnswerSubmissionView {
    pub player_id: u32,
    pub player_name: String,
    pub answer: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct FinalJeopardyView {
    pub category: String,
    pub question: String,
    pub answer: String,
}

pub fn build_game_view(
    state: &GameState,
    board_content: &BoardContent,
    include_answers: bool,
    submissions: &[AnswerSubmission],
) -> GameView {
    let players = state
        .players
        .iter()
        .map(|player| PlayerScoreView {
            id: player.id,
            name: player.name.clone(),
            score: player.score,
        })
        .collect();

    let board = state
        .rounds
        .iter()
        .map(|round| RoundView {
            name: round.name.clone(),
            categories: round
                .categories
                .iter()
                .map(|category| CategoryView {
                    title: category.title.clone(),
                    clues: category
                        .clues
                        .iter()
                        .map(|clue| ClueTileView {
                            label: clue.label.clone(),
                            value: clue.value,
                            answered: clue.answered,
                            daily_double: clue.is_daily_double,
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect();

    GameView {
        phase: state.phase.clone(),
        current_round: state.current_round,
        current_selector: state.current_selector.player_id(),
        players,
        board,
        active_clue: state.active_clue.as_ref().and_then(|active| {
            active_clue_view(active, state, board_content, include_answers, submissions)
        }),
        final_jeopardy: board_content
            .final_jeopardy
            .as_ref()
            .map(|final_clue| FinalJeopardyView {
                category: final_clue.category.clone(),
                question: final_clue.question.clone(),
                answer: if include_answers {
                    final_clue.answer.clone()
                } else {
                    String::new()
                },
            }),
    }
}

fn active_clue_view(
    active: &ActiveClue,
    state: &GameState,
    board_content: &BoardContent,
    include_answers: bool,
    submissions: &[AnswerSubmission],
) -> Option<ActiveClueView> {
    let tile = state
        .rounds
        .get(active.round_index)?
        .categories
        .get(active.category_index)?
        .clues
        .get(active.clue_index)?;
    let content = board_content
        .rounds
        .get(active.round_index)?
        .categories
        .get(active.category_index)?
        .clues
        .get(active.clue_index)?;

    Some(ActiveClueView {
        round_index: active.round_index,
        category_index: active.category_index,
        clue_index: active.clue_index,
        label: tile.label.clone(),
        value: tile.value,
        question: content.question.clone(),
        answer: if include_answers {
            content.answer.clone()
        } else {
            String::new()
        },
        attempted_player_ids: active.attempted_player_ids.clone(),
        submissions: if include_answers {
            submissions
                .iter()
                .map(|submission| AnswerSubmissionView {
                    player_id: submission.player_id,
                    player_name: submission.player_name.clone(),
                    answer: submission.answer.clone(),
                })
                .collect()
        } else {
            Vec::new()
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        content::models::{BoardCategory, BoardClue, BoardContent, BoardRound},
        domain::jeopardy::{
            ActiveClue, Category, Clue, GamePhase, PlayerState, RoundBoard, Selector,
        },
    };

    fn board_content() -> BoardContent {
        BoardContent {
            id: "generated".to_owned(),
            title: "Generated Board".to_owned(),
            rounds: vec![BoardRound {
                name: "Jeopardy".to_owned(),
                categories: vec![BoardCategory {
                    id: "movies".to_owned(),
                    title: "Movies".to_owned(),
                    clues: vec![BoardClue {
                        label: "$200".to_owned(),
                        question: "This movie has a shark.".to_owned(),
                        answer: "Jaws".to_owned(),
                        value: 200,
                        daily_double: false,
                    }],
                }],
            }],
            final_jeopardy: None,
        }
    }

    fn state() -> GameState {
        GameState {
            phase: GamePhase::ClueOpen,
            current_round: 0,
            current_selector: Selector::Moderator,
            players: vec![PlayerState {
                id: 7,
                name: "Ada".to_owned(),
                score: 0,
                final_wager: None,
                final_answer_correct: None,
            }],
            rounds: vec![RoundBoard {
                name: "Jeopardy".to_owned(),
                categories: vec![Category {
                    title: "Movies".to_owned(),
                    clues: vec![Clue {
                        id: 1,
                        label: "$200".to_owned(),
                        value: 200,
                        is_daily_double: false,
                        answered: false,
                    }],
                }],
            }],
            active_clue: Some(ActiveClue {
                round_index: 0,
                category_index: 0,
                clue_index: 0,
                selector_before_clue: Selector::Moderator,
                attempted_player_ids: vec![7],
                daily_double_wager: None,
                daily_double_player: None,
            }),
            final_jeopardy: None,
            final_reveal_order: Vec::new(),
        }
    }

    #[test]
    fn player_facing_view_redacts_answers_and_submissions() {
        let submissions = vec![AnswerSubmission {
            player_id: 7,
            player_name: "Ada".to_owned(),
            answer: "Jaws".to_owned(),
        }];

        let view = build_game_view(&state(), &board_content(), false, &submissions);
        let clue = view.active_clue.expect("active clue should be present");

        assert_eq!(clue.question, "This movie has a shark.");
        assert_eq!(clue.answer, "");
        assert!(clue.submissions.is_empty());
        assert_eq!(view.board[0].categories[0].clues.len(), 1);
    }
}
