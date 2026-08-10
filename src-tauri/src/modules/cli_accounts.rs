//! CLI Account Loading Module
//! Loads accounts from a JSON file containing [{email, refresh_token}] entries
//! and persists them using the existing account system.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Entry format in the accounts JSON file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountEntry {
    pub email: String,
    pub refresh_token: String,
}

/// Load account entries from a JSON file
pub fn load_entries_from_file(path: &Path) -> Result<Vec<AccountEntry>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read accounts file '{}': {}", path.display(), e))?;

    let entries: Vec<AccountEntry> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse accounts JSON: {}", e))?;

    Ok(entries)
}

/// Import a single account entry using refresh_token
/// Returns the email on success
pub async fn import_account_entry(entry: &AccountEntry) -> Result<String, String> {
    let service = crate::modules::account_service::AccountService::new(
        crate::modules::integration::SystemManager::Headless,
    );

    match service.add_account(&entry.refresh_token).await {
        Ok(account) => {
            tracing::info!("✅ Account loaded: {}", account.email);
            Ok(account.email)
        }
        Err(e) => {
            // If account already exists, try to just update the token
            if e.contains("already exists") {
                tracing::info!("♻️  Account already exists: {}, skipping", entry.email);
                Ok(entry.email.clone())
            } else {
                tracing::error!("❌ Failed to load account {}: {}", entry.email, e);
                Err(format!("Failed to load {}: {}", entry.email, e))
            }
        }
    }
}

/// Import all accounts from a file, with progress output
/// Returns (success_count, total_count)
pub async fn import_all_from_file(path: &Path) -> Result<(usize, usize), String> {
    let entries = load_entries_from_file(path)?;
    let total = entries.len();

    if total == 0 {
        return Err("No accounts found in file".to_string());
    }

    println!("📋 Found {} account(s) in {}", total, path.display());

    let mut success = 0;
    for (i, entry) in entries.iter().enumerate() {
        println!(
            "  [{}/{}] Loading {} ...",
            i + 1,
            total,
            entry.email
        );
        match import_account_entry(entry).await {
            Ok(_) => success += 1,
            Err(e) => {
                eprintln!("  ⚠️  {}", e);
            }
        }
    }

    println!("✅ Loaded {}/{} accounts successfully", success, total);
    Ok((success, total))
}
