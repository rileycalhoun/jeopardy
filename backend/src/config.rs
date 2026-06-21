pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub bind_address: String,
    pub bind_port: u16,
    pub frontend_origin: String,
    pub category_dir: String,
}

impl Config {
    pub fn from_env() -> Result<Self, std::num::ParseIntError> {
        let bind_port = std::env::var("BIND_PORT")
            .unwrap_or("8080".to_owned())
            .parse()?;

        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or("postgres://postgres:password@database:5432/docker".to_owned()),
            redis_url: std::env::var("REDIS_URL").unwrap_or("redis://redis:6379".to_owned()),
            bind_address: std::env::var("BIND_ADDRESS").unwrap_or("0.0.0.0".to_owned()),
            bind_port: bind_port,
            frontend_origin: std::env::var("FRONTEND_ORIGIN")
                .unwrap_or("http://localhost:5173".to_owned()),
            category_dir: std::env::var("CATEGORY_DIR").unwrap_or("./categories".to_owned()),
        })
    }
}
