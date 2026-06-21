use std::sync::Arc;

use redis::aio::ConnectionManager;
use sqlx::{Pool, Postgres};

use crate::{
    content::loader::CategoryPackLoader, realtime::hub::Hub, sessions::store::SessionStore,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub categories: CategoryPackLoader,
    /// Runtime session storage (Redis in production, memory in unit tests).
    pub sessions: Arc<dyn SessionStore>,
    /// Shared Redis connection used for publishing update events.
    pub redis: ConnectionManager,
    /// Fan-out of game updates to this instance's websocket clients.
    pub hub: Hub,
    /// Identifies this backend instance so it can skip its own pub/sub echoes.
    pub instance_id: String,
}
