use sqlx::{Pool, Postgres};

use crate::{content::loader::QuestionPackLoader, sessions::manager::SessionManager};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub question_packs: QuestionPackLoader,
    pub sessions: SessionManager,
}
