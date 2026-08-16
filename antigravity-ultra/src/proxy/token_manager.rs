use crate::account::import::AccountEntry;
use crate::models::{Account, TokenData};
use parking_lot::RwLock;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Token pool manager — manages multiple Google accounts for round-robin routing
pub struct TokenManager {
    accounts: RwLock<Vec<Account>>,
    current_index: AtomicUsize,
    auto_refresh: bool,
}

impl TokenManager {
    /// Create a new TokenManager and initialize all accounts
    pub async fn new(entries: Vec<AccountEntry>, auto_refresh: bool) -> Self {
        let mut accounts = Vec::new();

        for entry in &entries {
            let mut account = crate::account::import::entry_to_account(entry);

            // Try to refresh token immediately
            match crate::oauth::refresh_access_token(&entry.refresh_token, Some(&account.id)).await
            {
                Ok(token_resp) => {
                    let access_token_for_quota = token_resp.access_token.clone();
                    account.token = TokenData::new(
                        token_resp.access_token,
                        entry.refresh_token.clone(),
                        token_resp.expires_in,
                        Some(entry.email.clone()),
                        None,
                        None,
                        true,
                        token_resp.id_token,
                    );
                    tracing::info!(
                        "✅ [{}] Token refreshed (expires in {}s)",
                        entry.email,
                        token_resp.expires_in
                    );

                    // Fetch quota & project_id at startup so proxy routing works immediately
                    match crate::quota::fetch_quota(&access_token_for_quota, &entry.email, None).await {
                        Ok((quota_data, project_id)) => {
                            if let Some(ref pid) = project_id {
                                account.token.project_id = Some(pid.clone());
                                tracing::info!("   📋 [{}] Project ID: {}", entry.email, pid);
                            }
                            if !quota_data.is_forbidden {
                                account.update_quota(quota_data);
                            } else {
                                tracing::warn!("   ⚠️ [{}] Quota: FORBIDDEN (403)", entry.email);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("   ⚠️ [{}] Quota fetch failed: {}", entry.email, e);
                        }
                    }

                    accounts.push(account);
                }
                Err(e) => {
                    tracing::error!("❌ [{}] Token refresh failed: {}", entry.email, e);
                    accounts.push(account);
                }
            }
        }

        Self {
            accounts: RwLock::new(accounts),
            current_index: AtomicUsize::new(0),
            auto_refresh,
        }
    }

    /// Get the number of loaded accounts
    pub fn account_count(&self) -> usize {
        self.accounts.read().len()
    }

    /// Get next available account (round-robin)
    /// IMPORTANT: RwLockReadGuard must be dropped before any .await
    pub async fn get_next_account(&self) -> Option<Account> {
        // Phase 1: Select account under lock, then drop it
        let (account_opt, needs_refresh) = {
            let accounts = self.accounts.read();
            if accounts.is_empty() {
                return None;
            }

            let active_accounts: Vec<usize> = accounts
                .iter()
                .enumerate()
                .filter(|(_, a)| {
                    !a.disabled && !a.proxy_disabled && !a.token.access_token.is_empty()
                })
                .map(|(i, _)| i)
                .collect();

            if active_accounts.is_empty() {
                // Fallback: try any non-disabled account
                let fallback: Vec<usize> = accounts
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| !a.disabled)
                    .map(|(i, _)| i)
                    .collect();

                if fallback.is_empty() {
                    return None;
                }

                let idx = self.current_index.fetch_add(1, Ordering::Relaxed) % fallback.len();
                let account = accounts[fallback[idx]].clone();
                (Some(account), self.auto_refresh)
            } else {
                let idx =
                    self.current_index.fetch_add(1, Ordering::Relaxed) % active_accounts.len();
                let account = accounts[active_accounts[idx]].clone();
                let now = chrono::Utc::now().timestamp();
                let needs_refresh =
                    self.auto_refresh && account.token.expiry_timestamp < now + 900;
                (Some(account), needs_refresh)
            }
            // Guard dropped here
        };

        let mut account = account_opt?;

        // Phase 2: Refresh token if needed (no lock held)
        if needs_refresh {
            if let Ok(fresh) =
                crate::oauth::ensure_fresh_token(&account.token, Some(&account.id)).await
            {
                account.token = fresh.clone();
                // Update in pool
                let mut write_accounts = self.accounts.write();
                if let Some(acc) = write_accounts.iter_mut().find(|a| a.id == account.id) {
                    acc.token = fresh;
                }
            }
        }

        Some(account)
    }

    /// Get available model names from first account's quota
    pub fn get_available_models(&self) -> Vec<String> {
        let accounts = self.accounts.read();
        for acc in accounts.iter() {
            if let Some(ref quota) = acc.quota {
                return quota.models.iter().map(|m| m.name.clone()).collect();
            }
        }

        // Default models if no quota data available
        vec![
            "gemini-2.0-flash".to_string(),
            "gemini-2.5-pro".to_string(),
            "claude-sonnet-4".to_string(),
        ]
    }

    /// Get account summaries for admin API
    pub fn get_account_summaries(&self) -> Vec<Value> {
        let accounts = self.accounts.read();
        accounts
            .iter()
            .map(|acc| {
                json!({
                    "id": acc.id,
                    "email": acc.email,
                    "disabled": acc.disabled,
                    "proxy_disabled": acc.proxy_disabled,
                    "has_token": !acc.token.access_token.is_empty(),
                    "token_expires": acc.token.expiry_timestamp,
                    "last_used": acc.last_used,
                })
            })
            .collect()
    }

    /// Get all account quotas for monitoring
    pub fn get_all_quotas(&self) -> Vec<Value> {
        let accounts = self.accounts.read();
        accounts
            .iter()
            .filter_map(|acc| {
                acc.quota.as_ref().map(|q| {
                    json!({
                        "email": acc.email,
                        "subscription_tier": q.subscription_tier,
                        "is_forbidden": q.is_forbidden,
                        "models": q.models.iter().map(|m| {
                            json!({
                                "name": m.name,
                                "percentage": m.percentage,
                                "reset_time": m.reset_time,
                            })
                        }).collect::<Vec<_>>(),
                    })
                })
            })
            .collect()
    }

    /// Refresh all tokens (called by healthcheck)
    pub async fn refresh_all_tokens(&self) {
        // Collect account info under lock, then release
        let account_ids: Vec<(String, String, String)> = {
            let accounts = self.accounts.read();
            accounts
                .iter()
                .filter(|a| !a.disabled)
                .map(|a| {
                    (
                        a.id.clone(),
                        a.email.clone(),
                        a.token.refresh_token.clone(),
                    )
                })
                .collect()
        };

        for (id, email, refresh_token) in account_ids {
            match crate::oauth::refresh_access_token(&refresh_token, Some(&id)).await {
                Ok(token_resp) => {
                    let new_token = TokenData::new(
                        token_resp.access_token.clone(),
                        refresh_token,
                        token_resp.expires_in,
                        Some(email.clone()),
                        None,
                        None,
                        true,
                        token_resp.id_token,
                    );

                    // Update token in pool
                    {
                        let mut accounts = self.accounts.write();
                        if let Some(acc) = accounts.iter_mut().find(|a| a.id == id) {
                            acc.token = new_token;
                        }
                    } // Write lock dropped

                    // Fetch quota (no lock held)
                    match crate::quota::fetch_quota(&token_resp.access_token, &email, None).await {
                        Ok((quota_data, project_id)) => {
                            let mut accounts = self.accounts.write();
                            if let Some(acc) = accounts.iter_mut().find(|a| a.id == id) {
                                if quota_data.is_forbidden {
                                    acc.disabled = true;
                                    acc.disabled_reason =
                                        Some("403 Forbidden during healthcheck".to_string());
                                    tracing::warn!(
                                        "[Healthcheck] ⚠️ {} marked as forbidden",
                                        email
                                    );
                                } else {
                                    if acc
                                        .disabled_reason
                                        .as_ref()
                                        .map(|r| r.contains("healthcheck"))
                                        .unwrap_or(false)
                                    {
                                        acc.disabled = false;
                                        acc.disabled_reason = None;
                                    }
                                    if let Some(pid) = project_id {
                                        acc.token.project_id = Some(pid);
                                    }
                                    acc.update_quota(quota_data);
                                }
                            }
                            tracing::info!("[Healthcheck] ✅ {} refreshed & quota updated", email);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "[Healthcheck] ⚠️ {} quota fetch failed: {}",
                                email,
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("[Healthcheck] ❌ {} token refresh failed: {}", email, e);

                    if e.contains("invalid_grant") {
                        let mut accounts = self.accounts.write();
                        if let Some(acc) = accounts.iter_mut().find(|a| a.id == id) {
                            acc.disabled = true;
                            acc.disabled_reason =
                                Some(format!("invalid_grant during healthcheck: {}", e));
                            acc.disabled_at = Some(chrono::Utc::now().timestamp());
                        }
                    }
                }
            }
        }
    }
}
