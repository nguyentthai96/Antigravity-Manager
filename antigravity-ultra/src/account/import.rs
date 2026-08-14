use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Input format for antigravity_accounts.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountEntry {
    pub email: String,
    pub refresh_token: String,
}

/// Load accounts from a JSON file
/// Expected format: [{"email": "...", "refresh_token": "..."}, ...]
pub fn load_accounts_from_file(path: &std::path::Path) -> anyhow::Result<Vec<AccountEntry>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read accounts file '{}': {}", path.display(), e))?;

    let accounts: Vec<AccountEntry> = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse accounts JSON: {}", e))?;

    if accounts.is_empty() {
        tracing::warn!("Accounts file is empty: {}", path.display());
    }

    Ok(accounts)
}

/// Default accounts file path (relative to CWD or antigravity-cli directory)
pub fn default_accounts_path() -> PathBuf {
    // Try common locations
    let candidates = [
        PathBuf::from("antigravity_accounts.json"),
        PathBuf::from("antigravity-cli/antigravity_accounts.json"),
        PathBuf::from("../antigravity-cli/antigravity_accounts.json"),
    ];

    for p in &candidates {
        if p.exists() {
            return p.clone();
        }
    }

    // Return first as default (will fail with proper error message)
    candidates[0].clone()
}

/// Convert AccountEntry to full Account model
pub fn entry_to_account(entry: &AccountEntry) -> crate::models::Account {
    let id = uuid::Uuid::new_v4().to_string();
    let token = crate::models::TokenData::new(
        String::new(),          // access_token (will be refreshed)
        entry.refresh_token.clone(),
        0,                      // expires_in
        Some(entry.email.clone()),
        None,                   // project_id
        None,                   // session_id
        true,                   // is_gcp_tos
        None,                   // id_token
    );
    crate::models::Account::new(id, entry.email.clone(), token)
}
