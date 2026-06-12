use std::sync::Arc;

use redis::aio::ConnectionManager;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    app::build_app, config::Config, content::loader::QuestionPackLoader, realtime::hub::Hub,
    sessions::redis_store::RedisSessionStore, state::AppState,
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
pub mod realtime;
pub mod sessions;
pub mod state;

fn generate_instance_id() -> String {
    let bytes: [u8; 8] = rand::random();
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
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

    // Redis holds active runtime sessions and the update event channel.
    let redis_client = match redis::Client::open(config.redis_url.as_str()) {
        Ok(client) => client,
        Err(err) => {
            error!("invalid redis url: {}", err);
            return Err(err.into());
        }
    };
    let redis = match ConnectionManager::new(redis_client.clone()).await {
        Ok(conn) => {
            info!("connected to redis at {}", &config.redis_url);
            conn
        }
        Err(err) => {
            error!(
                "could not connect to redis at {}: {}",
                &config.redis_url, err
            );
            return Err(err.into());
        }
    };

    let state = AppState {
        pool,
        question_packs: QuestionPackLoader::new(&config.question_pack_dir),
        sessions: Arc::new(RedisSessionStore::new(redis.clone())),
        redis,
        hub: Hub::default(),
        instance_id: generate_instance_id(),
    };

    // Rebroadcast updates from other backend instances to local websockets.
    realtime::spawn_event_listener(state.clone(), redis_client);

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
