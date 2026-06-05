use sqlx::{Pool, Postgres};

use crate::players::models::{PlayerRow, PlayerSummary};

pub async fn insert_player(
    pool: &Pool<Postgres>,
    game_id: i64,
    display_name: String,
) -> Result<PlayerRow, sqlx::Error> {
    sqlx::query_as::<_, PlayerRow>(
        r#"
        INSERT INTO players (game_id, display_name)
        VALUES ($1, $2)
        RETURNING id, game_id, display_name
        "#,
    )
    .bind(game_id)
    .bind(display_name)
    .fetch_one(pool)
    .await
}

pub async fn list_players_for_game(
    pool: &Pool<Postgres>,
    game_id: i64,
) -> Result<Vec<PlayerSummary>, sqlx::Error> {
    sqlx::query_as::<_, PlayerSummary>(
        r#"
        SELECT id, display_name
        FROM players
        WHERE game_id = $1
        ORDER BY id
        "#,
    )
    .bind(game_id)
    .fetch_all(pool)
    .await
}
