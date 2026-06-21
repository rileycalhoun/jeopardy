use sqlx::{Pool, Postgres};

use crate::games::models::{GameData, GameRow};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AdminTokenRow {
    pub game_id: i64,
    pub token_hash: String,
    pub label: String,
}

pub async fn insert_game(
    pool: &Pool<Postgres>,
    game_id: String,
    admin_code: i32,
    player_code: i32,
) -> Result<Option<GameData>, sqlx::Error> {
    sqlx::query_as::<_, GameData>(
        r#"
                INSERT INTO games (game_id, admin_code, player_code)
                VALUES ($1, $2, $3)
                ON CONFLICT DO NOTHING
                RETURNING game_id, admin_code, player_code
                "#,
    )
    .bind(game_id)
    .bind(admin_code)
    .bind(player_code)
    .fetch_optional(pool)
    .await
}

pub async fn find_game_by_player_code(
    pool: &Pool<Postgres>,
    player_code: i32,
) -> Result<Option<GameRow>, sqlx::Error> {
    sqlx::query_as::<_, GameRow>(
        r#"
        SELECT id, game_id, admin_code, player_code
        FROM games
        WHERE player_code = $1
        "#,
    )
    .bind(player_code)
    .fetch_optional(pool)
    .await
}

pub async fn find_game_by_admin_code(
    pool: &Pool<Postgres>,
    admin_code: i32,
) -> Result<Option<GameRow>, sqlx::Error> {
    sqlx::query_as::<_, GameRow>(
        r#"
        SELECT id, game_id, admin_code, player_code
        FROM games
        WHERE admin_code = $1
        "#,
    )
    .bind(admin_code)
    .fetch_optional(pool)
    .await
}

pub async fn mark_game_started(
    pool: &Pool<Postgres>,
    game_id: i64,
    content_id: String,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE games
        SET status = 'in_progress',
            question_pack_id = $2,
            started_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(game_id)
    .bind(content_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_game_completed(pool: &Pool<Postgres>, game_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE games
        SET status = 'completed',
            completed_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(game_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn insert_admin_token(
    pool: &Pool<Postgres>,
    game_id: i64,
    token_hash: String,
    label: String,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO admin_tokens (game_id, token_hash, label)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(game_id)
    .bind(token_hash)
    .bind(label)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn find_active_admin_token_by_hash(
    pool: &Pool<Postgres>,
    token_hash: &str,
) -> Result<Option<AdminTokenRow>, sqlx::Error> {
    sqlx::query_as::<_, AdminTokenRow>(
        r#"
        SELECT game_id, token_hash, label
        FROM admin_tokens
        WHERE token_hash = $1
          AND revoked_at IS NULL
        "#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
}
