use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    app::build_app, config::Config, content::loader::QuestionPackLoader,
    sessions::manager::SessionManager, state::AppState,
};

pub mod app;
pub mod config;
pub mod content;
pub mod db;
pub mod domain;
pub mod error;
pub mod games;
pub mod http;
pub mod moderation;
pub mod players;
pub mod sessions;
pub mod state;

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

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(err) => {
            error!(?err, "could not parse config");
            return Err(err.into());
        }
    };

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await;

    if let Err(err) = pool {
        error!("could not connect to database");
        return Err(err.into());
    }

    // Valid since checked above
    let pool = pool.unwrap();
    let state = AppState {
        pool: pool,
        question_packs: QuestionPackLoader::new(&config.question_pack_dir),
        sessions: SessionManager::default(),
    };
    let handler = build_app(state, &config);
    info!(
        "listening on {}:{}",
        &config.bind_address, &config.bind_port
    );

    // Keep bind and serve errors separate so the logs show which phase failed.
    match TcpListener::bind((config.bind_address.as_str(), config.bind_port)).await {
        Ok(listener) => match axum::serve(listener, handler).await {
            Ok(_) => Ok(()),
            Err(err) => {
                error!("Could not serve app: {}", err);
                Err(err.into())
            }
        },
        Err(err) => {
            error!(
                "Could not bind to address {}:{}: {}",
                &config.bind_address, &config.bind_port, err
            );

            Err(err.into())
        }
    }
}
