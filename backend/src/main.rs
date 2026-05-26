use axum::{
    Json, Router,
    extract::State,
    http::{Method, StatusCode, header},
    routing::post,
};
use rand::{RngExt, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    pool: Pool<Postgres>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Loading `.env` is optional; only treat parse/load failures as fatal.
    if let Err(err) = dotenvy::dotenv() {
        if !err.not_found() {
            eprintln!("could not load dotenv file: {}", err);
            return Err(err.into());
        }
    }

    // Initialize logging before any other startup checks so early failures are visible.
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    // Fall back to local development defaults when env vars are not set.
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or("postgres://postgres:password@database:5432/docker".to_owned());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await;

    if let Err(err) = pool {
        error!("could not connect to database with url: {database_url}");
        return Err(err.into());
    }

    // Valid since checked above
    let pool = pool.unwrap();
    let state = AppState { pool: pool };

    let bind_address = std::env::var("BIND_ADDRESS").unwrap_or("0.0.0.0".to_owned());
    let bind_port_str = std::env::var("BIND_PORT").unwrap_or("8080".to_owned());

    // Exit on invalid ports instead of silently choosing a different one.
    let bind_port: u16 = match bind_port_str.parse() {
        Ok(port) => port,
        Err(err) => {
            error!("invalid BIND_PORT '{}': {}", bind_port_str, err);
            return Err(err.into());
        }
    };

    let cors = CorsLayer::new()
        .allow_origin(["http://localhost:5173".parse().unwrap()])
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    let root_handler = Router::new()
        .route("/games/new", post(create_new_game))
        .layer(cors)
        .with_state(state);

    info!("listening on {}:{}", &bind_address, &bind_port);

    // Keep bind and serve errors separate so the logs show which phase failed.
    match TcpListener::bind((bind_address.as_str(), bind_port)).await {
        Ok(listener) => match axum::serve(listener, root_handler).await {
            Ok(_) => Ok(()),
            Err(err) => {
                error!("Could not serve app: {}", err);
                Err(err.into())
            }
        },
        Err(err) => {
            error!(
                "Could not bind to address {}:{}: {}",
                bind_address, bind_port, err
            );

            Err(err.into())
        }
    }
}

#[derive(Serialize, Deserialize)]
struct GameData {
    game_id: String,
    admin_code: i32,
    player_code: i32,
}

async fn create_new_game(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<GameData>), StatusCode> {
    for _ in 0..10 {
        // Generate new game_id
        let game_id: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        // Generate new admin_code
        let admin_code: i32 = rand::rng().random_range(100_000..1_000_000);

        // Generate new player_code
        let player_code: i32 = rand::rng().random_range(100_000..1_000_000);

        let pool = &state.pool;
        let inserted = sqlx::query_as!(
            GameData,
            r#"
            INSERT INTO games (game_id, admin_code, player_code)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            RETURNING game_id, admin_code, player_code
            "#,
            game_id,
            admin_code,
            player_code,
        )
        .fetch_optional(pool)
        .await;

        match inserted {
            Ok(Some(game)) => {
                return Ok((
                    StatusCode::OK,
                    Json(GameData {
                        game_id: game.game_id,
                        admin_code: game.admin_code,
                        player_code: game.player_code,
                    }),
                ));
            }
            Ok(None) => continue,
            Err(err) => {
                error!("failed to insert game: {}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    error!("could not generate unique values after 10 tries");
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
