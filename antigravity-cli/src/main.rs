mod db;
mod device;
mod models;
mod oauth;
mod process;
mod protobuf;

use clap::Parser;
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the JSON file containing the account list
    #[arg(short, long)]
    accounts_file: String,

    /// Email of the account to switch to
    #[arg(short, long)]
    email: String,
    
    /// Optional GCP project ID to inject
    #[arg(short, long)]
    project_id: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Args::parse();

    // 1. Read the accounts JSON file
    let accounts_json = fs::read_to_string(&args.accounts_file)
        .map_err(|e| format!("Failed to read accounts file {}: {}", args.accounts_file, e))?;
    
    let accounts: Vec<models::AccountItem> = serde_json::from_str(&accounts_json)
        .map_err(|e| format!("Failed to parse accounts JSON: {}", e))?;

    // 2. Find the target account
    let target_account = accounts
        .into_iter()
        .find(|acc| acc.email == args.email)
        .ok_or_else(|| format!("Account with email {} not found in the list.", args.email))?;

    println!("Found account: {}", target_account.email);

    // 3. Refresh token
    let token_res = oauth::refresh_access_token(&target_account.refresh_token).await?;

    let new_refresh_token = token_res.refresh_token.unwrap_or(target_account.refresh_token);
    let expiry = chrono::Local::now().timestamp() + token_res.expires_in;

    // 4. Close Antigravity if running
    if process::is_antigravity_running() {
        process::close_antigravity(10)?;
    }

    // 5. Generate and inject new device profile
    let storage_path = device::get_storage_path()?;
    let new_profile = device::generate_profile();
    device::write_profile(&storage_path, &new_profile)?;

    // 6. Inject Token into State DB
    let db_path = db::get_db_path()?;
    let is_gcp_tos = true; // Hardcoded true or pass via argument if needed
    
    db::inject_token(
        &db_path,
        &token_res.access_token,
        &new_refresh_token,
        expiry,
        &target_account.email,
        is_gcp_tos,
        args.project_id.as_deref(),
    )?;

    // 7. Restart Antigravity
    process::start_antigravity()?;

    println!("Account switch to {} completed successfully!", args.email);

    Ok(())
}
