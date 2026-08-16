use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod account;
mod config;
mod constants;
mod error;
mod logger;
mod models;
mod oauth;
mod proxy;
mod quota;
mod user_token;
mod utils;

#[derive(Parser)]
#[command(name = "antigravity-ultra")]
#[command(about = "Standalone headless proxy for Antigravity — zero Tauri dependencies")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the proxy server
    Start {
        /// Port to listen on
        #[arg(short, long, default_value = "8045")]
        port: u16,

        /// Path to accounts JSON file (email + refresh_token list)
        #[arg(short, long)]
        accounts: Option<PathBuf>,

        /// Listen on all interfaces (0.0.0.0) instead of localhost only
        #[arg(long)]
        lan: bool,

        /// Auto-generate a user token on startup if none exists
        #[arg(long)]
        auto_token: bool,

        /// Specify a fixed API key (e.g. sk-mykey123). Used with --auto-token.
        #[arg(long)]
        api_key: Option<String>,

        /// Enable auto-refresh for expired tokens
        #[arg(long, default_value = "true")]
        auto_refresh: bool,

        /// Healthcheck interval in seconds (0 to disable)
        #[arg(long, default_value = "600")]
        healthcheck_interval: u64,
    },

    /// Manage user tokens (API keys)
    Token {
        #[command(subcommand)]
        action: TokenAction,
    },

    /// Manage accounts
    Accounts {
        #[command(subcommand)]
        action: AccountAction,
    },

    /// Show system information
    Info,
}

#[derive(Subcommand)]
enum TokenAction {
    /// Create a new user token
    Create {
        /// Username for the token
        #[arg(short, long)]
        username: String,

        /// Expiration type: never, day, week, month
        #[arg(short, long, default_value = "never")]
        expires: String,

        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// List all user tokens
    List,

    /// Revoke (delete) a user token
    Revoke {
        /// Token ID to revoke
        id: String,
    },
}

#[derive(Subcommand)]
enum AccountAction {
    /// List loaded accounts with status
    List {
        /// Path to accounts JSON file
        #[arg(short, long)]
        accounts: Option<PathBuf>,
    },

    /// Run a one-shot healthcheck on all accounts
    Healthcheck {
        /// Path to accounts JSON file
        #[arg(short, long)]
        accounts: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logger
    logger::init_logger();

    // Initialize user token database
    if let Err(e) = user_token::init_db() {
        tracing::warn!("Failed to initialize user token DB: {}", e);
    }

    match cli.command {
        Commands::Start {
            port,
            accounts,
            lan,
            auto_token,
            api_key,
            auto_refresh,
            healthcheck_interval,
        } => {
            cmd_start(port, accounts, lan, auto_token, api_key, auto_refresh, healthcheck_interval).await?;
        }
        Commands::Token { action } => match action {
            TokenAction::Create {
                username,
                expires,
                description,
            } => {
                cmd_token_create(&username, &expires, description.as_deref())?;
            }
            TokenAction::List => {
                cmd_token_list()?;
            }
            TokenAction::Revoke { id } => {
                cmd_token_revoke(&id)?;
            }
        },
        Commands::Accounts { action } => match action {
            AccountAction::List { accounts } => {
                cmd_accounts_list(accounts).await?;
            }
            AccountAction::Healthcheck { accounts } => {
                cmd_accounts_healthcheck(accounts).await?;
            }
        },
        Commands::Info => {
            cmd_info();
        }
    }

    Ok(())
}

// ──────────────────────────────────────────────────────────────
// Command implementations
// ──────────────────────────────────────────────────────────────

async fn cmd_start(
    port: u16,
    accounts_path: Option<PathBuf>,
    lan: bool,
    auto_token: bool,
    api_key: Option<String>,
    auto_refresh: bool,
    healthcheck_interval: u64,
) -> anyhow::Result<()> {
    tracing::info!("🚀 Antigravity Ultra v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("   Port: {}", port);
    tracing::info!("   LAN: {}", lan);
    tracing::info!("   Auto-refresh: {}", auto_refresh);
    tracing::info!(
        "   Healthcheck interval: {}s",
        if healthcheck_interval > 0 {
            format!("{}", healthcheck_interval)
        } else {
            "disabled".to_string()
        }
    );

    // 1. Load accounts
    let accounts = if let Some(path) = &accounts_path {
        tracing::info!("   Accounts file: {}", path.display());
        account::import::load_accounts_from_file(path)?
    } else {
        // Try default path
        let default_path = account::import::default_accounts_path();
        if default_path.exists() {
            tracing::info!(
                "   Accounts file: {} (auto-detected)",
                default_path.display()
            );
            account::import::load_accounts_from_file(&default_path)?
        } else {
            tracing::warn!("   No accounts file found. Proxy will start with empty pool.");
            Vec::new()
        }
    };

    tracing::info!("   Loaded {} accounts", accounts.len());
    for acc in &accounts {
        tracing::info!("     📧 {}", acc.email);
    }

    // 2. Auto-generate token if needed
    if auto_token {
        let tokens = user_token::list_tokens().unwrap_or_default();
        if tokens.is_empty() {
            match user_token::create_token(
                "admin".to_string(),
                "never".to_string(),
                Some("Auto-generated on first start".to_string()),
                0,
                None,
                None,
                None,
                api_key.clone(),
            ) {
                Ok(token) => {
                    tracing::info!("🔑 Auto-generated API key: {}", token.token);
                }
                Err(e) => {
                    tracing::warn!("Failed to auto-generate token: {}", e);
                }
            }
        } else if let Some(ref key) = api_key {
            // Check if the desired key already exists
            let key_exists = tokens.iter().any(|t| t.token == *key);
            if !key_exists {
                tracing::info!("🔑 Existing tokens found but not matching --api-key, keeping existing tokens.");
                tracing::info!("   Current key: {}", tokens[0].token);
            }
        }
    }

    // 3. Build and start proxy server
    let proxy_config = proxy::ProxyStartConfig {
        port,
        lan,
        auto_refresh,
        healthcheck_interval,
        accounts,
    };

    proxy::start_proxy_server(proxy_config).await?;

    Ok(())
}

fn cmd_token_create(
    username: &str,
    expires: &str,
    description: Option<&str>,
) -> anyhow::Result<()> {
    let token = user_token::create_token(
        username.to_string(),
        expires.to_string(),
        description.map(String::from),
        0,
        None,
        None,
        None,
        None,
    )
    .map_err(|e| anyhow::anyhow!(e))?;

    println!("✅ Token created successfully!");
    println!("   ID:       {}", token.id);
    println!("   Token:    {}", token.token);
    println!("   Username: {}", token.username);
    println!("   Expires:  {}", token.expires_type);

    Ok(())
}

fn cmd_token_list() -> anyhow::Result<()> {
    let tokens = user_token::list_tokens().map_err(|e| anyhow::anyhow!(e))?;

    if tokens.is_empty() {
        println!("No tokens found. Create one with: antigravity-ultra token create --username <name>");
        return Ok(());
    }

    println!(
        "{:<36} {:<40} {:<15} {:<10} {:<10}",
        "ID", "TOKEN", "USERNAME", "EXPIRES", "ENABLED"
    );
    println!("{}", "-".repeat(111));

    for t in &tokens {
        println!(
            "{:<36} {:<40} {:<15} {:<10} {:<10}",
            t.id,
            &t.token[..t.token.len().min(40)],
            t.username,
            t.expires_type,
            if t.enabled { "✓" } else { "✗" }
        );
    }

    println!("\nTotal: {} tokens", tokens.len());
    Ok(())
}

fn cmd_token_revoke(id: &str) -> anyhow::Result<()> {
    user_token::delete_token(id).map_err(|e| anyhow::anyhow!(e))?;
    println!("✅ Token {} revoked successfully", id);
    Ok(())
}

async fn cmd_accounts_list(accounts_path: Option<PathBuf>) -> anyhow::Result<()> {
    let path = accounts_path.unwrap_or_else(account::import::default_accounts_path);

    if !path.exists() {
        println!("❌ Accounts file not found: {}", path.display());
        return Ok(());
    }

    let accounts = account::import::load_accounts_from_file(&path)?;

    println!("📋 Accounts from: {}", path.display());
    println!("{:<30} {:<50}", "EMAIL", "REFRESH TOKEN (prefix)");
    println!("{}", "-".repeat(80));

    for acc in &accounts {
        let token_prefix = if acc.refresh_token.len() > 30 {
            format!("{}...", &acc.refresh_token[..30])
        } else {
            acc.refresh_token.clone()
        };
        println!("{:<30} {:<50}", acc.email, token_prefix);
    }

    println!("\nTotal: {} accounts", accounts.len());
    Ok(())
}

async fn cmd_accounts_healthcheck(accounts_path: Option<PathBuf>) -> anyhow::Result<()> {
    let path = accounts_path.unwrap_or_else(account::import::default_accounts_path);

    if !path.exists() {
        println!("❌ Accounts file not found: {}", path.display());
        return Ok(());
    }

    let accounts = account::import::load_accounts_from_file(&path)?;
    println!("🏥 Running healthcheck on {} accounts...\n", accounts.len());

    for acc in &accounts {
        print!("  📧 {} ... ", acc.email);

        // Try to refresh token
        match oauth::refresh_access_token(&acc.refresh_token, None).await {
            Ok(token_resp) => {
                println!("✅ Token OK (expires in {}s)", token_resp.expires_in);

                // Try to fetch quota
                match quota::fetch_quota(&token_resp.access_token, &acc.email, None).await {
                    Ok((quota_data, _project_id)) => {
                        if quota_data.is_forbidden {
                            println!("    ⚠️  Quota: FORBIDDEN (403)");
                        } else {
                            println!("    📊 Quota: {} models available", quota_data.models.len());
                            for m in &quota_data.models {
                                let bar = "█".repeat((m.percentage / 10) as usize);
                                let empty = "░".repeat(10 - (m.percentage / 10) as usize);
                                println!(
                                    "       {} {}{} {}%",
                                    m.name, bar, empty, m.percentage
                                );
                            }
                        }
                    }
                    Err(e) => {
                        println!("    ⚠️  Quota fetch failed: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ FAILED: {}", e);
            }
        }
    }

    Ok(())
}

fn cmd_info() {
    println!("Antigravity Ultra v{}", env!("CARGO_PKG_VERSION"));
    println!("  Standalone headless proxy — zero Tauri dependencies");
    println!();
    println!("Data directory: {}", config::get_data_dir().display());
    println!("User-Agent:     {}", constants::get_user_agent());
    println!();
    println!("Usage:");
    println!("  antigravity-ultra start --accounts accounts.json --port 8045");
    println!("  antigravity-ultra token create --username admin");
    println!("  antigravity-ultra accounts healthcheck --accounts accounts.json");
}
