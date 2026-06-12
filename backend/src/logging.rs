use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_LOG_FILTER: &str = "info";

pub fn init() {
    let filter = match std::env::var(EnvFilter::DEFAULT_ENV) {
        Ok(value) => EnvFilter::try_new(value).unwrap_or_else(|err| {
            eprintln!("invalid RUST_LOG value; using {DEFAULT_LOG_FILTER}: {err}");
            EnvFilter::new(DEFAULT_LOG_FILTER)
        }),
        Err(std::env::VarError::NotPresent) => EnvFilter::new(DEFAULT_LOG_FILTER),
        Err(err) => {
            eprintln!("could not read RUST_LOG; using {DEFAULT_LOG_FILTER}: {err}");
            EnvFilter::new(DEFAULT_LOG_FILTER)
        }
    };

    let format = std::env::var("LOG_FORMAT")
        .unwrap_or_else(|_| "pretty".to_owned())
        .to_ascii_lowercase();

    match format.as_str() {
        "json" => tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::layer()
                    .json()
                    .with_target(true)
                    .with_file(true)
                    .with_line_number(true)
                    .flatten_event(true),
            )
            .init(),
        "pretty" => tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::layer()
                    .pretty()
                    .with_target(true)
                    .with_file(true)
                    .with_line_number(true),
            )
            .init(),
        invalid => {
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .pretty()
                        .with_target(true)
                        .with_file(true)
                        .with_line_number(true),
                )
                .init();
            tracing::warn!(
                log_format = invalid,
                fallback = "pretty",
                "invalid LOG_FORMAT value"
            );
        }
    }
}
