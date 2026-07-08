use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize a `tracing` subscriber that emits structured logs.
///
/// Behavior is controlled by the `RUST_LOG` environment variable. If unset,
/// defaults to `info`.
///
/// # Panics
/// Panics if a tracing subscriber has already been initialized in the same
/// process. Call this exactly once, early in `main()`.
pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer().with_target(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}

/// Initialize logging with a specific level, useful in tests and binaries.
///
/// # Panics
/// Panics if a tracing subscriber has already been initialized.
pub fn init_logging_with_level(level: &str) {
    let filter = EnvFilter::try_new(level).unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer().with_target(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}
