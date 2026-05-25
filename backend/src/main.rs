use axum::{
    Json, Router,
    http::{Method, StatusCode, header},
    routing::get,
};
use rand::{RngExt, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

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
        .allow_methods([Method::GET])
        .allow_headers([header::CONTENT_TYPE]);

    let root_handler = Router::new()
        .route("/games/new", get(create_new_game))
        .layer(cors);
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
    admin_code: usize,
    player_code: usize,
}

async fn create_new_game() -> (StatusCode, Json<GameData>) {
    // Generate new game_id
    let game_id: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    // Generate new admin_code
    let admin_code: usize = rand::rng().random_range(100_000..1_000_000);

    // Generate new player_code
    let player_code: usize = rand::rng().random_range(100_000..1_000_000);

    // TODO: Check for any duplicates in SQLx

    (
        StatusCode::OK,
        Json(GameData {
            game_id,
            admin_code,
            player_code,
        }),
    )
}
