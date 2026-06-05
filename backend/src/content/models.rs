use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QuestionPack {
    pub id: String,
    pub title: String,
    pub rounds: Vec<PackRound>,
    pub final_jeopardy: Option<PackFinalJeopardy>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PackRound {
    pub name: String,
    pub categories: Vec<PackCategory>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PackCategory {
    pub title: String,
    pub clues: Vec<PackClue>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PackClue {
    pub label: String,
    pub question: String,
    pub answer: String,
    pub value: i32,
    #[serde(default)]
    pub daily_double: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PackFinalJeopardy {
    pub category: String,
    pub question: String,
    pub answer: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct QuestionPackSummary {
    pub id: String,
    pub title: String,
}

impl QuestionPack {
    pub fn validate(&self) -> Result<(), String> {
        validate_id(&self.id, "pack id")?;
        validate_non_empty(&self.title, "pack title")?;

        if self.rounds.is_empty() && self.final_jeopardy.is_none() {
            return Err("pack must include at least one round or final clue".to_owned());
        }

        for round in &self.rounds {
            validate_non_empty(&round.name, "round name")?;
            if round.categories.is_empty() {
                return Err(format!("round '{}' must include categories", round.name));
            }

            for category in &round.categories {
                validate_non_empty(&category.title, "category title")?;
                if category.clues.is_empty() {
                    return Err(format!("category '{}' must include clues", category.title));
                }

                for clue in &category.clues {
                    validate_non_empty(&clue.label, "clue label")?;
                    validate_non_empty(&clue.question, "clue question")?;
                    validate_non_empty(&clue.answer, "clue answer")?;
                    if clue.value <= 0 {
                        return Err(format!("clue '{}' must have a positive value", clue.label));
                    }
                }
            }
        }

        if let Some(final_jeopardy) = &self.final_jeopardy {
            validate_non_empty(&final_jeopardy.category, "final category")?;
            validate_non_empty(&final_jeopardy.question, "final question")?;
            validate_non_empty(&final_jeopardy.answer, "final answer")?;
        }

        Ok(())
    }

    pub fn summary(&self) -> QuestionPackSummary {
        QuestionPackSummary {
            id: self.id.clone(),
            title: self.title.clone(),
        }
    }
}

fn validate_non_empty(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{field} must not be empty"));
    }

    Ok(())
}

fn validate_id(value: &str, field: &str) -> Result<(), String> {
    validate_non_empty(value, field)?;
    if !value
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        return Err(format!(
            "{field} may only contain lowercase letters, numbers, hyphens, and underscores"
        ));
    }

    Ok(())
}
