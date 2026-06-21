use sqlx::{Pool, Postgres};

use crate::players::models::{PlayerRow, PlayerSummary};

pub async fn insert_player(
    pool: &Pool<Postgres>,
    game_id: i64,
    display_name: String,
    token_hash: String,
) -> Result<PlayerRow, sqlx::Error> {
    sqlx::query_as::<_, PlayerRow>(
        r#"
        INSERT INTO players (game_id, display_name, token_hash)
        VALUES ($1, $2, $3)
        RETURNING id, game_id, display_name
        "#,
    )
    .bind(game_id)
    .bind(display_name)
    .bind(token_hash)
    .fetch_one(pool)
    .await
}

pub async fn find_player_by_token_hash(
    pool: &Pool<Postgres>,
    token_hash: &str,
) -> Result<Option<PlayerRow>, sqlx::Error> {
    sqlx::query_as::<_, PlayerRow>(
        r#"
        SELECT id, game_id, display_name
        FROM players
        WHERE token_hash = $1
        "#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
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
