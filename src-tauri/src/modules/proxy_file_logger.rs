/// Proxy Request/Response File Logger
///
/// Writes structured JSONL logs for every proxy request to daily-rotating files.
/// Files are stored in `~/.antigravity_tools/logs/proxy_requests.log.YYYY-MM-DD`
///
/// Each line is a JSON object with request metadata, request body summary, response summary,
/// token usage, timing, and error info.

use crate::proxy::monitor::ProxyRequestLog;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

/// Controls whether proxy file logging is active
static FILE_LOGGING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Custom log directory override (empty = default)
static mut CUSTOM_LOG_DIR: Option<PathBuf> = None;

pub fn set_file_logging_enabled(enabled: bool) {
    FILE_LOGGING_ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn is_file_logging_enabled() -> bool {
    FILE_LOGGING_ENABLED.load(Ordering::Relaxed)
}

/// Set a custom log directory (call before starting the proxy)
/// Safety: Must be called only once during initialization, before any concurrent access
pub fn set_custom_log_dir(dir: PathBuf) {
    unsafe {
        CUSTOM_LOG_DIR = Some(dir);
    }
}

fn get_proxy_log_dir() -> Result<PathBuf, String> {
    // Check custom dir first
    let custom = unsafe { CUSTOM_LOG_DIR.as_ref() };
    if let Some(dir) = custom {
        if !dir.exists() {
            fs::create_dir_all(dir)
                .map_err(|e| format!("Failed to create custom log directory: {}", e))?;
        }
        return Ok(dir.clone());
    }

    // Default: ~/.antigravity_tools/logs/
    let data_dir = crate::modules::account::get_data_dir()?;
    let log_dir = data_dir.join("logs");
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)
            .map_err(|e| format!("Failed to create log directory: {}", e))?;
    }
    Ok(log_dir)
}

fn get_today_log_path() -> Result<PathBuf, String> {
    let log_dir = get_proxy_log_dir()?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    Ok(log_dir.join(format!("proxy_requests.log.{}", today)))
}

/// Format a ProxyRequestLog into a compact JSONL entry for file logging
fn format_log_entry(log: &ProxyRequestLog) -> String {
    let timestamp = chrono::Local::now().to_rfc3339();

    // Build a structured log entry
    let mut entry = serde_json::json!({
        "timestamp": timestamp,
        "id": log.id,
        "method": log.method,
        "url": log.url,
        "status": log.status,
        "duration_ms": log.duration,
    });

    let obj = entry.as_object_mut().unwrap();

    if let Some(ref model) = log.model {
        obj.insert("model".to_string(), serde_json::json!(model));
    }
    if let Some(ref mapped_model) = log.mapped_model {
        obj.insert("mapped_model".to_string(), serde_json::json!(mapped_model));
    }
    if let Some(ref protocol) = log.protocol {
        obj.insert("protocol".to_string(), serde_json::json!(protocol));
    }
    if let Some(ref account_email) = log.account_email {
        obj.insert("account".to_string(), serde_json::json!(account_email));
    }
    if let Some(ref client_ip) = log.client_ip {
        obj.insert("client_ip".to_string(), serde_json::json!(client_ip));
    }
    if let Some(ref username) = log.username {
        obj.insert("username".to_string(), serde_json::json!(username));
    }

    // Token usage
    if log.input_tokens.is_some() || log.output_tokens.is_some() || log.cached_tokens.is_some() {
        let mut tokens = serde_json::Map::new();
        if let Some(input) = log.input_tokens {
            tokens.insert("input".to_string(), serde_json::json!(input));
        }
        if let Some(output) = log.output_tokens {
            tokens.insert("output".to_string(), serde_json::json!(output));
        }
        if let Some(cached) = log.cached_tokens {
            tokens.insert("cached".to_string(), serde_json::json!(cached));
        }
        obj.insert("tokens".to_string(), serde_json::Value::Object(tokens));
    }

    // Request body (truncated for file log)
    if let Some(ref body) = log.request_body {
        let truncated = if body.len() > 2000 {
            format!("{}...[truncated, {} bytes total]", &body[..2000], body.len())
        } else {
            body.clone()
        };
        obj.insert("request_body".to_string(), serde_json::json!(truncated));
    }

    // Response body (truncated for file log)
    if let Some(ref body) = log.response_body {
        let truncated = if body.len() > 2000 {
            format!("{}...[truncated, {} bytes total]", &body[..2000], body.len())
        } else {
            body.clone()
        };
        obj.insert("response_body".to_string(), serde_json::json!(truncated));
    }

    // Error
    if let Some(ref error) = log.error {
        obj.insert("error".to_string(), serde_json::json!(error));
    }

    serde_json::to_string(&entry).unwrap_or_else(|_| "{{\"error\":\"serialization_failed\"}}".to_string())
}

/// Write a ProxyRequestLog to the daily log file (append mode)
/// This should be called from a blocking context (tokio::task::spawn_blocking)
pub fn write_log_to_file(log: &ProxyRequestLog) {
    if !is_file_logging_enabled() {
        return;
    }

    let log_path = match get_today_log_path() {
        Ok(path) => path,
        Err(e) => {
            tracing::error!("Failed to get proxy log path: {}", e);
            return;
        }
    };

    let entry = format_log_entry(log);

    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        Ok(mut file) => {
            if let Err(e) = writeln!(file, "{}", entry) {
                tracing::error!("Failed to write proxy log: {}", e);
            }
        }
        Err(e) => {
            tracing::error!("Failed to open proxy log file {:?}: {}", log_path, e);
        }
    }
}

/// Cleanup proxy request log files older than specified days
pub fn cleanup_old_proxy_logs(days_to_keep: u64) -> Result<u32, String> {
    let log_dir = get_proxy_log_dir()?;
    let cutoff = chrono::Local::now() - chrono::Duration::days(days_to_keep as i64);
    let cutoff_str = cutoff.format("%Y-%m-%d").to_string();
    let mut deleted = 0u32;

    let entries = fs::read_dir(&log_dir)
        .map_err(|e| format!("Failed to read log directory: {}", e))?;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        // Match files like proxy_requests.log.2026-08-01
        if let Some(date_str) = name.strip_prefix("proxy_requests.log.") {
            if date_str < cutoff_str.as_str() {
                if let Err(e) = fs::remove_file(entry.path()) {
                    tracing::warn!("Failed to delete old proxy log {:?}: {}", name, e);
                } else {
                    tracing::info!("Deleted old proxy request log: {}", name);
                    deleted += 1;
                }
            }
        }
    }

    Ok(deleted)
}
