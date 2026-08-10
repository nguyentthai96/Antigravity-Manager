//! Bulk Token Import Tool
//! Accepts a list of {username, refresh_token} entries,
//! imports each as an account and generates sk-... user API keys.

use serde::{Deserialize, Serialize};

use crate::modules::account_service::AccountService;
use crate::modules::logger;
use crate::modules::user_token_db;

/// Single entry in the bulk import request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BulkTokenEntry {
    pub username: String,
    pub refresh_token: String,
}

/// Result for a single entry after processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkTokenResult {
    pub username: String,
    pub email: Option<String>,
    pub api_key: Option<String>,
    pub account_id: Option<String>,
    pub status: String, // "success" | "failed"
    pub error: Option<String>,
}

/// Aggregated response for the entire bulk import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkTokenResponse {
    pub total: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub results: Vec<BulkTokenResult>,
}

/// Process a list of {username, refresh_token} entries:
/// 1. For each entry, create/upsert the account via OAuth refresh flow
/// 2. Generate an sk-... user API key for the username
/// 3. Collect results with per-entry success/failure tracking
pub async fn process_bulk_import(
    entries: Vec<BulkTokenEntry>,
    service: &AccountService,
) -> BulkTokenResponse {
    let total = entries.len();
    let mut results = Vec::with_capacity(total);
    let mut success_count = 0usize;
    let mut failed_count = 0usize;

    for (idx, entry) in entries.iter().enumerate() {
        logger::log_info(&format!(
            "[BulkImport] Processing {}/{}: username={}",
            idx + 1,
            total,
            entry.username
        ));

        let result = process_single_entry(entry, service).await;

        match &result.status[..] {
            "success" => success_count += 1,
            _ => failed_count += 1,
        }

        results.push(result);
    }

    logger::log_info(&format!(
        "[BulkImport] Complete: total={}, success={}, failed={}",
        total, success_count, failed_count
    ));

    BulkTokenResponse {
        total,
        success_count,
        failed_count,
        results,
    }
}

/// Process a single entry: add account + generate user token
async fn process_single_entry(entry: &BulkTokenEntry, service: &AccountService) -> BulkTokenResult {
    // Step 1: Add/upsert the account using the refresh_token
    let account = match service.add_account(&entry.refresh_token).await {
        Ok(acc) => acc,
        Err(e) => {
            logger::log_warn(&format!(
                "[BulkImport] Failed to add account for {}: {}",
                entry.username, e
            ));
            return BulkTokenResult {
                username: entry.username.clone(),
                email: None,
                api_key: None,
                account_id: None,
                status: "failed".to_string(),
                error: Some(format!("Account creation failed: {}", e)),
            };
        }
    };

    // Step 2: Check if user already has an active token
    let existing_token = match user_token_db::list_tokens() {
        Ok(tokens) => tokens
            .into_iter()
            .find(|t| t.username == entry.username && t.enabled),
        Err(_) => None,
    };

    let api_key = if let Some(existing) = existing_token {
        // User already has an active token, return it
        logger::log_info(&format!(
            "[BulkImport] User {} already has an active token, skipping creation",
            entry.username
        ));
        existing.token
    } else {
        // Step 3: Generate a new sk-... user API key
        match user_token_db::create_token(
            entry.username.clone(),
            "never".to_string(), // No expiry by default
            Some(format!("Auto-generated for {}", account.email)),
            0,    // Unlimited IPs
            None, // No curfew
            None,
            None,
        ) {
            Ok(token) => {
                logger::log_info(&format!(
                    "[BulkImport] Generated API key for user {}: sk-...{}",
                    entry.username,
                    &token.token[token.token.len().saturating_sub(6)..]
                ));
                token.token
            }
            Err(e) => {
                logger::log_warn(&format!(
                    "[BulkImport] Failed to create user token for {}: {}",
                    entry.username, e
                ));
                return BulkTokenResult {
                    username: entry.username.clone(),
                    email: Some(account.email),
                    api_key: None,
                    account_id: Some(account.id),
                    status: "failed".to_string(),
                    error: Some(format!("Token creation failed: {}", e)),
                };
            }
        }
    };

    BulkTokenResult {
        username: entry.username.clone(),
        email: Some(account.email),
        api_key: Some(api_key),
        account_id: Some(account.id),
        status: "success".to_string(),
        error: None,
    }
}
