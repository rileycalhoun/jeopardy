use sqlx::{PgPool, Row};

use super::repository;

#[sqlx::test]
async fn persists_started_game_metadata(pool: PgPool) -> sqlx::Result<()> {
    let created_game = repository::insert_game(&pool, "GAME01".to_owned(), 123_456, 654_321)
        .await?
        .expect("game should be inserted");

    let stored_game = repository::find_game_by_player_code(&pool, created_game.player_code)
        .await?
        .expect("game should be queryable by player code");

    let initial_row = sqlx::query(
        r#"
        SELECT status, question_pack_id, started_at IS NOT NULL AS has_started_at
        FROM games
        WHERE id = $1
        "#,
    )
    .bind(stored_game.id)
    .fetch_one(&pool)
    .await?;

    assert_eq!(initial_row.get::<String, _>("status"), "lobby");
    assert_eq!(
        initial_row.get::<Option<String>, _>("question_pack_id"),
        None
    );
    assert!(!initial_row.get::<bool, _>("has_started_at"));

    repository::mark_game_started(&pool, stored_game.id, "classic".to_owned()).await?;

    let row = sqlx::query(
        r#"
        SELECT status, question_pack_id, started_at IS NOT NULL AS has_started_at
        FROM games
        WHERE id = $1
        "#,
    )
    .bind(stored_game.id)
    .fetch_one(&pool)
    .await?;

    assert_eq!(row.get::<String, _>("status"), "in_progress");
    assert_eq!(
        row.get::<Option<String>, _>("question_pack_id"),
        Some("classic".to_owned())
    );
    assert!(row.get::<bool, _>("has_started_at"));

    Ok(())
}

#[sqlx::test]
async fn persists_active_admin_token_lookup(pool: PgPool) -> sqlx::Result<()> {
    let created_game = repository::insert_game(&pool, "GAME02".to_owned(), 222_222, 333_333)
        .await?
        .expect("game should be inserted");

    let stored_game = repository::find_game_by_player_code(&pool, created_game.player_code)
        .await?
        .expect("game should be queryable by player code");

    assert!(
        repository::find_active_admin_token_by_hash(&pool, "token-hash")
            .await?
            .is_none(),
        "lookup should miss before the token is inserted"
    );

    repository::insert_admin_token(
        &pool,
        stored_game.id,
        "token-hash".to_owned(),
        "host laptop".to_owned(),
    )
    .await?;

    let token = repository::find_active_admin_token_by_hash(&pool, "token-hash")
        .await?
        .expect("token should be queryable by hash");

    assert_eq!(token.game_id, stored_game.id);
    assert_eq!(token.token_hash, "token-hash");
    assert_eq!(token.label, "host laptop");

    Ok(())
}
