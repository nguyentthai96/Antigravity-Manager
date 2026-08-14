use std::fs;
use std::path::PathBuf;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

struct LocalTimer;

impl tracing_subscriber::fmt::time::FormatTime for LocalTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = chrono::Local::now();
        write!(w, "{}", now.to_rfc3339())
    }
}

pub fn get_log_dir() -> PathBuf {
    let data_dir = crate::config::get_data_dir();
    let log_dir = data_dir.join("logs");

    if !log_dir.exists() {
        let _ = fs::create_dir_all(&log_dir);
    }

    log_dir
}

/// Initialize the log system
pub fn init_logger() {
    let _ = tracing_log::LogTracer::init();

    let log_dir = get_log_dir();

    let file_appender = tracing_appender::rolling::daily(&log_dir, "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let console_layer = fmt::Layer::new()
        .with_target(false)
        .with_thread_ids(false)
        .with_level(true)
        .with_timer(LocalTimer);

    let file_layer = fmt::Layer::new()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_level(true)
        .with_timer(LocalTimer);

    let filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let _ = tracing_subscriber::registry()
        .with(filter_layer)
        .with(console_layer)
        .with(file_layer)
        .try_init();

    // Leak guard to ensure logging persists until process exit
    std::mem::forget(_guard);

    info!("Log system initialized (Console + File)");
}

/// Log info message
pub fn log_info(message: &str) {
    info!("{}", message);
}

/// Log warn message
pub fn log_warn(message: &str) {
    warn!("{}", message);
}

/// Log error message
pub fn log_error(message: &str) {
    error!("{}", message);
}
