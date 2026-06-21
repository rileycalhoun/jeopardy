use std::sync::Arc;

use redis::aio::ConnectionManager;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::{
    app::build_app, config::Config, content::loader::CategoryPackLoader, realtime::hub::Hub,
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
pub mod logging;
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
    // Loading `.env` happens before tracing so it can configure the subscriber.
    if let Err(err) = dotenvy::dotenv() {
        if !err.not_found() {
            eprintln!("could not load dotenv file: {err}");
            return Err(err.into());
        }
    }

    logging::init();

    let config = match Config::from_env() {
        Ok(config) => {
            info!(
                bind_address = %config.bind_address,
                bind_port = config.bind_port,
                category_dir = %config.category_dir,
                "configuration loaded"
            );
            config
        }
        Err(err) => {
            error!(?err, "could not parse config");
            return Err(err.into());
        }
    };

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await;

    let pool = match pool {
        Ok(pool) => {
            info!("connected to database");
            pool
        }
        Err(err) => {
            error!(?err, "could not connect to database");
            return Err(err.into());
        }
    };

    // Redis holds active runtime sessions and the update event channel.
    let redis_client = match redis::Client::open(config.redis_url.as_str()) {
        Ok(client) => client,
        Err(err) => {
            error!(?err, "invalid redis URL");
            return Err(err.into());
        }
    };
    let redis = match ConnectionManager::new(redis_client.clone()).await {
        Ok(conn) => {
            info!("connected to redis");
            conn
        }
        Err(err) => {
            error!(?err, "could not connect to redis");
            return Err(err.into());
        }
    };

    let state = AppState {
        pool,
        categories: CategoryPackLoader::new(&config.category_dir),
        sessions: Arc::new(RedisSessionStore::new(redis.clone())),
        redis,
        hub: Hub::default(),
        instance_id: generate_instance_id(),
    };

    // Rebroadcast updates from other backend instances to local websockets.
    realtime::spawn_event_listener(state.clone(), redis_client);

    let handler = build_app(state, &config);
    info!(
        bind_address = %config.bind_address,
        bind_port = config.bind_port,
        "starting HTTP server"
    );

    // Keep bind and serve errors separate so the logs show which phase failed.
    match TcpListener::bind((config.bind_address.as_str(), config.bind_port)).await {
        Ok(listener) => match axum::serve(listener, handler).await {
            Ok(_) => Ok(()),
            Err(err) => {
                error!(?err, "HTTP server failed");
                Err(err.into())
            }
        },
        Err(err) => {
            error!(
                ?err,
                bind_address = %config.bind_address,
                bind_port = config.bind_port,
                "could not bind HTTP listener"
            );

            Err(err.into())
        }
    }
}
