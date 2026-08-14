//! Autonomous Quota Health Monitor
//!
//! Background task that periodically checks quota status of ALL accounts
//! and re-activates accounts whose quota has recovered. This ensures the
//! proxy can self-heal when all accounts are temporarily exhausted.
//!
//! Key behaviors:
//! - Runs periodically (default: every 10 minutes)
//! - Can be woken up immediately when TokenManager detects all accounts exhausted
//! - Smart sleeping: when all accounts are exhausted, sleeps until the earliest
//!   known reset_time, then re-checks
//! - Only re-enables accounts that were disabled by quota_protection (not manually disabled)

use std::path::PathBuf;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use crate::models::QuotaHealthCheckConfig;
use crate::proxy::TokenManager;

/// Start the quota health monitor background task.
///
/// This spawns a tokio task that:
/// 1. Periodically checks quota status of all accounts on disk
/// 2. Re-activates accounts whose quota has recovered
/// 3. Sleeps intelligently until the earliest reset_time when all accounts are exhausted
///
/// Returns a JoinHandle for the spawned task.
pub fn start_health_monitor(
    token_manager: Arc<TokenManager>,
    config: QuotaHealthCheckConfig,
    cancel_token: CancellationToken,
) -> tokio::task::JoinHandle<()> {
    let notify = token_manager.get_health_check_notify();

    tokio::spawn(async move {
        tracing::info!(
            "🏥 [QuotaHealthMonitor] Started (interval={}s, smart_sleep={}, buffer={}s, max_sleep={}s)",
            config.interval_seconds,
            config.smart_sleep_enabled,
            config.reset_buffer_seconds,
            config.max_sleep_seconds,
        );

        let interval_duration = std::time::Duration::from_secs(config.interval_seconds);
        let mut interval = tokio::time::interval(interval_duration);
        // Skip the first immediate tick (let the system stabilize)
        interval.tick().await;

        loop {
            // Wait for either: periodic tick, on-demand signal, or cancellation
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    tracing::info!("🏥 [QuotaHealthMonitor] Received cancel signal, shutting down");
                    break;
                }
                _ = interval.tick() => {
                    tracing::debug!("🏥 [QuotaHealthMonitor] Periodic health check triggered");
                }
                _ = notify.notified() => {
                    tracing::info!("🏥 [QuotaHealthMonitor] On-demand health check triggered (all accounts exhausted)");
                    // Reset the interval so the next periodic check is a full interval away
                    interval.reset();
                }
            }

            // Execute the health check cycle
            let result = run_health_check_cycle(
                &token_manager,
                &config,
                &cancel_token,
            )
            .await;

            match result {
                HealthCheckResult::AllRecovered(count) => {
                    tracing::info!(
                        "🏥 [QuotaHealthMonitor] ✅ {} account(s) recovered and re-activated",
                        count
                    );
                }
                HealthCheckResult::SomeRecovered { recovered, still_exhausted } => {
                    tracing::info!(
                        "🏥 [QuotaHealthMonitor] ⚡ {} recovered, {} still exhausted",
                        recovered,
                        still_exhausted
                    );
                }
                HealthCheckResult::AllExhausted { earliest_reset_secs } => {
                    if config.smart_sleep_enabled {
                        if let Some(sleep_secs) = earliest_reset_secs {
                            let sleep_secs = sleep_secs
                                .saturating_add(config.reset_buffer_seconds)
                                .min(config.max_sleep_seconds);

                            if sleep_secs > 0 {
                                tracing::info!(
                                    "🏥 [QuotaHealthMonitor] 💤 All accounts exhausted. Smart sleeping for {}s ({}m) until quota reset...",
                                    sleep_secs,
                                    sleep_secs / 60
                                );

                                // Sleep with cancellation support and on-demand wake-up
                                tokio::select! {
                                    _ = cancel_token.cancelled() => {
                                        tracing::info!("🏥 [QuotaHealthMonitor] Cancel during smart sleep");
                                        break;
                                    }
                                    _ = tokio::time::sleep(std::time::Duration::from_secs(sleep_secs)) => {
                                        tracing::info!("🏥 [QuotaHealthMonitor] ⏰ Woke up from smart sleep, re-checking...");
                                    }
                                    _ = notify.notified() => {
                                        tracing::info!("🏥 [QuotaHealthMonitor] 🔔 Woke up early from smart sleep by on-demand signal");
                                    }
                                }
                            }
                        } else {
                            tracing::warn!(
                                "🏥 [QuotaHealthMonitor] ⚠️ All accounts exhausted but no reset_time available, will retry at next interval"
                            );
                        }
                    }
                }
                HealthCheckResult::NoAccounts => {
                    tracing::debug!("🏥 [QuotaHealthMonitor] No accounts found on disk");
                }
                HealthCheckResult::Error(e) => {
                    tracing::warn!("🏥 [QuotaHealthMonitor] Health check error: {}", e);
                }
            }
        }

        tracing::info!("🏥 [QuotaHealthMonitor] Shut down");
    })
}

/// Result of a single health check cycle
enum HealthCheckResult {
    /// All accounts have recovered quota
    AllRecovered(usize),
    /// Some accounts recovered, some still exhausted
    SomeRecovered {
        recovered: usize,
        still_exhausted: usize,
    },
    /// All accounts are still exhausted
    AllExhausted {
        /// Seconds until the earliest known reset time (None if unknown)
        earliest_reset_secs: Option<u64>,
    },
    /// No accounts found on disk
    NoAccounts,
    /// Error during health check
    Error(String),
}

/// Execute one complete health check cycle:
/// 1. Load all accounts from disk (including disabled ones)
/// 2. For each account, check if quota has recovered
/// 3. Re-activate recovered accounts in the TokenManager
async fn run_health_check_cycle(
    token_manager: &Arc<TokenManager>,
    config: &QuotaHealthCheckConfig,
    cancel_token: &CancellationToken,
) -> HealthCheckResult {
    let data_dir = token_manager.get_data_dir().clone();
    let accounts_dir = data_dir.join("accounts");

    if !accounts_dir.exists() {
        return HealthCheckResult::NoAccounts;
    }

    // Read all account JSON files from disk
    let entries = match std::fs::read_dir(&accounts_dir) {
        Ok(e) => e,
        Err(e) => return HealthCheckResult::Error(format!("Failed to read accounts dir: {}", e)),
    };

    let mut account_files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();

    if account_files.is_empty() {
        return HealthCheckResult::NoAccounts;
    }

    account_files.sort(); // deterministic order

    // [NEW] Also check which accounts are QuotaExhausted in memory (RateLimitTracker)
    let rate_limit_exhausted = token_manager
        .get_rate_limit_tracker()
        .get_quota_exhausted_account_ids();

    tracing::debug!(
        "🏥 [QuotaHealthMonitor] Scanning {} account files, {} in-memory QuotaExhausted entries",
        account_files.len(),
        rate_limit_exhausted.len()
    );

    let mut recovered_count = 0;
    let mut still_exhausted_count = 0;
    let mut earliest_reset_timestamp: Option<i64> = None;
    let now = chrono::Utc::now().timestamp();

    // Process accounts with concurrency limit of 3 to avoid API rate limiting
    let semaphore = Arc::new(tokio::sync::Semaphore::new(3));
    let mut tasks = Vec::new();

    for path in account_files {
        if cancel_token.is_cancelled() {
            break;
        }

        let sem = semaphore.clone();
        let path_clone = path.clone();
        let cancel = cancel_token.clone();
        let exhausted_ids = rate_limit_exhausted.clone();

        tasks.push(tokio::spawn(async move {
            let _permit = match sem.acquire().await {
                Ok(p) => p,
                Err(_) => return AccountCheckResult::Error("Semaphore closed".to_string()),
            };

            if cancel.is_cancelled() {
                return AccountCheckResult::Skipped;
            }

            check_single_account(&path_clone, &exhausted_ids).await
        }));
    }

    // Collect results
    for task in tasks {
        match task.await {
            Ok(AccountCheckResult::Recovered {
                account_id,
                email,
            }) => {
                // Re-activate this account in the TokenManager
                match token_manager.reload_account(&account_id).await {
                    Ok(()) => {
                        // Also clear QuotaExhausted entries from rate limit tracker
                        token_manager
                            .get_rate_limit_tracker()
                            .clear_quota_exhausted_for_account(&account_id);

                        tracing::info!(
                            "🏥 [QuotaHealthMonitor] ✅ Re-activated account: {} ({})",
                            email,
                            account_id
                        );
                        recovered_count += 1;
                    }
                    Err(e) => {
                        tracing::warn!(
                            "🏥 [QuotaHealthMonitor] Failed to reload account {}: {}",
                            account_id,
                            e
                        );
                    }
                }
            }
            Ok(AccountCheckResult::StillExhausted { reset_timestamp, .. }) => {
                still_exhausted_count += 1;
                if let Some(ts) = reset_timestamp {
                    match earliest_reset_timestamp {
                        Some(current) if ts < current => earliest_reset_timestamp = Some(ts),
                        None => earliest_reset_timestamp = Some(ts),
                        _ => {}
                    }
                }
            }
            Ok(AccountCheckResult::Healthy { .. }) => {
                // Account is already healthy, nothing to do
            }
            Ok(AccountCheckResult::ManuallyDisabled) => {
                // Skip manually disabled accounts
            }
            Ok(AccountCheckResult::Skipped) => {}
            Ok(AccountCheckResult::Error(e)) => {
                tracing::debug!("🏥 [QuotaHealthMonitor] Account check error: {}", e);
            }
            Err(e) => {
                tracing::warn!("🏥 [QuotaHealthMonitor] Task join error: {}", e);
            }
        }
    }

    // Calculate earliest reset time relative to now
    let earliest_reset_secs = earliest_reset_timestamp.map(|ts| {
        if ts > now {
            (ts - now) as u64
        } else {
            0
        }
    });

    if recovered_count > 0 && still_exhausted_count == 0 {
        HealthCheckResult::AllRecovered(recovered_count)
    } else if recovered_count > 0 {
        HealthCheckResult::SomeRecovered {
            recovered: recovered_count,
            still_exhausted: still_exhausted_count,
        }
    } else if still_exhausted_count > 0 {
        HealthCheckResult::AllExhausted {
            earliest_reset_secs,
        }
    } else {
        // All accounts are healthy or manually disabled
        HealthCheckResult::AllRecovered(0)
    }
}

/// Result of checking a single account
enum AccountCheckResult {
    /// Account's quota has recovered and can be re-activated
    Recovered { account_id: String, email: String },
    /// Account is still exhausted
    StillExhausted {
        account_id: String,
        email: String,
        reset_timestamp: Option<i64>,
    },
    /// Account is already healthy (not disabled by quota protection)
    Healthy { account_id: String },
    /// Account is manually disabled (not by quota protection)
    ManuallyDisabled,
    /// Skipped (cancelled)
    Skipped,
    /// Error checking account
    Error(String),
}

/// Check a single account's quota status
/// `rate_limit_exhausted_ids` - accounts currently blocked by QuotaExhausted in RateLimitTracker
async fn check_single_account(path: &PathBuf, rate_limit_exhausted_ids: &[String]) -> AccountCheckResult {
    // Read account JSON from disk
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return AccountCheckResult::Error(format!("Read error: {}", e)),
    };

    let account_json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => return AccountCheckResult::Error(format!("Parse error: {}", e)),
    };

    let account_id = match path.file_stem().and_then(|s| s.to_str()) {
        Some(id) => id.to_string(),
        None => return AccountCheckResult::Error("No account_id from filename".to_string()),
    };

    let email = account_json
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // Check if account is manually disabled (not by quota protection)
    let is_disabled = account_json
        .get("disabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if is_disabled {
        return AccountCheckResult::ManuallyDisabled;
    }

    // Check if proxy_disabled and the reason
    let proxy_disabled = account_json
        .get("proxy_disabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let proxy_disabled_reason = account_json
        .get("proxy_disabled_reason")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Check protected_models list
    let protected_models: Vec<String> = account_json
        .get("protected_models")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // If account is not disabled by quota protection and has no protected models,
    // ALSO check if it's in the rate_limit_exhausted list (in-memory QuotaExhausted)
    let is_rate_limit_exhausted = rate_limit_exhausted_ids.contains(&account_id);

    if !proxy_disabled && protected_models.is_empty() && !is_rate_limit_exhausted {
        return AccountCheckResult::Healthy { account_id };
    }

    // Only process accounts disabled by quota protection or those with protected models
    if proxy_disabled && !proxy_disabled_reason.contains("quota") {
        return AccountCheckResult::ManuallyDisabled;
    }

    // Try to check quota by refreshing the token and calling the API
    let refresh_token = match account_json.get("refresh_token").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return AccountCheckResult::Error("No refresh_token".to_string()),
    };

    // Refresh the OAuth token
    let token_result =
        crate::modules::oauth::refresh_access_token(&refresh_token, Some(&account_id)).await;

    let access_token = match token_result {
        Ok(resp) => resp.access_token,
        Err(e) => {
            tracing::debug!(
                "🏥 [QuotaHealthMonitor] Token refresh failed for {}: {}",
                email,
                e
            );
            // If token refresh fails, this account can't be recovered right now
            return AccountCheckResult::StillExhausted {
                account_id,
                email,
                reset_timestamp: None,
            };
        }
    };

    // Fetch quota data
    let quota_result =
        crate::modules::quota::fetch_quota(&access_token, &email, Some(&account_id)).await;

    let (quota_data, _project_id) = match quota_result {
        Ok(data) => data,
        Err(e) => {
            tracing::debug!(
                "🏥 [QuotaHealthMonitor] Quota fetch failed for {}: {:?}",
                email,
                e
            );
            return AccountCheckResult::StillExhausted {
                account_id,
                email,
                reset_timestamp: None,
            };
        }
    };

    // Load quota protection config to know the threshold
    let quota_config = crate::modules::config::load_app_config()
        .map(|cfg| cfg.quota_protection)
        .unwrap_or_default();

    let threshold = quota_config.threshold_percentage as i32;

    // Check if any monitored model has recovered its quota
    let mut has_recovered_models = false;
    let mut all_still_exhausted = true;
    let mut earliest_reset: Option<i64> = None;

    for model in &quota_data.models {
        // Check if this model was previously protected
        let std_id = crate::proxy::common::model_mapping::normalize_to_standard_id(&model.name);
        let is_monitored = if let Some(ref std) = std_id {
            protected_models.contains(std) || quota_config.monitored_models.contains(std)
        } else {
            false
        };

        if !is_monitored {
            continue;
        }

        if model.percentage >= threshold {
            has_recovered_models = true;
            all_still_exhausted = false;
            tracing::debug!(
                "🏥 [QuotaHealthMonitor] Model {} recovered to {}% (threshold: {}%)",
                model.name,
                model.percentage,
                threshold
            );
        } else if model.percentage == 0 {
            // Parse reset_time to get the timestamp
            if !model.reset_time.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&model.reset_time) {
                    let ts = dt.timestamp();
                    match earliest_reset {
                        Some(current) if ts < current => earliest_reset = Some(ts),
                        None => earliest_reset = Some(ts),
                        _ => {}
                    }
                }
            }
        }
    }

    if has_recovered_models {
        // Update account on disk to clear protection
        let mut updated_json = account_json.clone();
        if proxy_disabled && proxy_disabled_reason.contains("quota") {
            updated_json["proxy_disabled"] = serde_json::Value::Bool(false);
            updated_json["proxy_disabled_reason"] = serde_json::Value::Null;
            updated_json["proxy_disabled_at"] = serde_json::Value::Null;
        }

        // Update protected_models: remove recovered models
        if let Some(arr) = updated_json
            .get_mut("protected_models")
            .and_then(|v| v.as_array_mut())
        {
            arr.retain(|m| {
                if let Some(model_name) = m.as_str() {
                    // Check if this model has recovered
                    let recovered = quota_data.models.iter().any(|qm| {
                        let std = crate::proxy::common::model_mapping::normalize_to_standard_id(
                            &qm.name,
                        );
                        std.as_deref() == Some(model_name) && qm.percentage >= threshold
                    });
                    !recovered // Keep models that have NOT recovered
                } else {
                    true
                }
            });
        }

        // Save updated account JSON
        if let Ok(json_str) = serde_json::to_string_pretty(&updated_json) {
            let _ = std::fs::write(path, json_str);
        }

        AccountCheckResult::Recovered { account_id, email }
    } else if all_still_exhausted {
        AccountCheckResult::StillExhausted {
            account_id,
            email,
            reset_timestamp: earliest_reset,
        }
    } else {
        AccountCheckResult::Healthy { account_id }
    }
}
