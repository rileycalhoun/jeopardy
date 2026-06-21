use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const POINT_VALUES: [i32; 5] = [200, 400, 600, 800, 1000];

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CategoryPack {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(flatten, default)]
    pub metadata: HashMap<String, Value>,
    pub questions: Vec<CategoryQuestion>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CategoryQuestion {
    pub points: i32,
    pub question: String,
    pub answer: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CategorySummary {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BoardContent {
    pub id: String,
    pub title: String,
    pub rounds: Vec<BoardRound>,
    pub final_jeopardy: Option<BoardFinalJeopardy>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BoardRound {
    pub name: String,
    pub categories: Vec<BoardCategory>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BoardCategory {
    pub id: String,
    pub title: String,
    pub clues: Vec<BoardClue>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BoardClue {
    pub label: String,
    pub question: String,
    pub answer: String,
    pub value: i32,
    #[serde(default)]
    pub daily_double: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BoardFinalJeopardy {
    pub category: String,
    pub question: String,
    pub answer: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CategoryValidationError {
    EmptyField(&'static str),
    InvalidId { field: &'static str },
    NoQuestions,
    InvalidPointValue(i32),
    MissingPointValue(i32),
}

impl fmt::Display for CategoryValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(formatter, "{field} must not be empty"),
            Self::InvalidId { field } => write!(
                formatter,
                "{field} may only contain lowercase letters, numbers, hyphens, and underscores"
            ),
            Self::NoQuestions => write!(formatter, "category must include questions"),
            Self::InvalidPointValue(points) => write!(formatter, "invalid point value {points}"),
            Self::MissingPointValue(points) => {
                write!(formatter, "no questions for required point value {points}")
            }
        }
    }
}

impl CategoryPack {
    pub fn validate(&self) -> Result<(), CategoryValidationError> {
        validate_id(&self.id, "category id")?;
        validate_non_empty(&self.title, "category title")?;

        if self.questions.is_empty() {
            return Err(CategoryValidationError::NoQuestions);
        }

        let mut counts = HashMap::new();
        for question in &self.questions {
            question.validate()?;
            *counts.entry(question.points).or_insert(0_usize) += 1;
        }

        for points in POINT_VALUES {
            if !counts.contains_key(&points) {
                return Err(CategoryValidationError::MissingPointValue(points));
            }
        }

        Ok(())
    }

    pub fn summary(&self) -> CategorySummary {
        CategorySummary {
            id: self.id.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
        }
    }

    pub fn questions_for_points(&self, points: i32) -> Vec<&CategoryQuestion> {
        self.questions
            .iter()
            .filter(|question| question.points == points)
            .collect()
    }
}

impl CategoryQuestion {
    pub fn validate(&self) -> Result<(), CategoryValidationError> {
        if !POINT_VALUES.contains(&self.points) {
            return Err(CategoryValidationError::InvalidPointValue(self.points));
        }
        validate_non_empty(&self.question, "question")?;
        validate_non_empty(&self.answer, "answer")?;
        Ok(())
    }
}

fn validate_non_empty(value: &str, field: &'static str) -> Result<(), CategoryValidationError> {
    if value.trim().is_empty() {
        return Err(CategoryValidationError::EmptyField(field));
    }

    Ok(())
}

fn validate_id(value: &str, field: &'static str) -> Result<(), CategoryValidationError> {
    validate_non_empty(value, field)?;
    if !value
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        return Err(CategoryValidationError::InvalidId { field });
    }

    Ok(())
}

#[cfg(test)]
mod category_tests {
    use super::*;

    fn question(points: i32) -> CategoryQuestion {
        CategoryQuestion {
            points,
            question: format!("{points} point clue"),
            answer: format!("{points} point answer"),
        }
    }

    fn valid_category() -> CategoryPack {
        CategoryPack {
            id: "video_games".to_owned(),
            title: "Video Games".to_owned(),
            description: Some("Games and gaming history.".to_owned()),
            metadata: Default::default(),
            questions: POINT_VALUES.into_iter().map(question).collect(),
        }
    }

    #[test]
    fn category_requires_all_classic_jeopardy_point_values_for_gameplay() {
        let mut category = valid_category();
        category.questions.retain(|question| question.points != 1000);

        let err = category
            .validate()
            .expect_err("missing point value should be rejected");

        assert!(
            err.to_string()
                .contains("no questions for required point value 1000")
        );
    }

    #[test]
    fn category_rejects_invalid_point_values() {
        let mut category = valid_category();
        category.questions.push(question(750));

        let err = category
            .validate()
            .expect_err("invalid point value should be rejected");

        assert!(err.to_string().contains("invalid point value 750"));
    }
}
