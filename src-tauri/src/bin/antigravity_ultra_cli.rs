//! Antigravity Ultra CLI
//! Standalone CLI for proxying API tokens for external tools.
//!
//! This binary reuses the Antigravity Manager's proxy infrastructure
//! (TokenManager, AxumServer, UserToken DB) to provide a headless
//! proxy service that external tools (Claude Code, Cursor, etc.)
//! can call via generated API keys (sk-* tokens).

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;

/// Antigravity Ultra — Proxy API Token for External AI Tools
#[derive(Parser)]
#[command(name = "antigravity-ultra")]
#[command(version, about = "Proxy API token for external tools via Antigravity Ultra")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the proxy server with accounts from a JSON file
    Start {
        /// Port to listen on
        #[arg(short, long, default_value_t = 8045)]
        port: u16,

        /// Path to accounts JSON file [{email, refresh_token}, ...]
        #[arg(short, long, default_value = "antigravity_accounts.json")]
        accounts: PathBuf,

        /// Allow LAN access (bind 0.0.0.0 instead of 127.0.0.1)
        #[arg(long, default_value_t = false)]
        lan: bool,

        /// Auto-create a default User Token if none exists
        #[arg(long, default_value_t = true)]
        auto_token: bool,

        /// Custom log directory for proxy request/response logs
        #[arg(long)]
        log_dir: Option<PathBuf>,
    },

    /// Manage User Tokens (API keys for external tools)
    Token {
        #[command(subcommand)]
        action: TokenAction,
    },

    /// Show connection info and example usage
    Info,
}

#[derive(Subcommand)]
enum TokenAction {
    /// Create a new API key for an external tool
    Create {
        /// Username/label for the token (e.g. "claude-code", "cursor")
        #[arg(short, long)]
        username: String,

        /// Expiration type: day, week, month, never
        #[arg(short, long, default_value = "never")]
        expires: String,

        /// Max number of IPs allowed (0 = unlimited)
        #[arg(long, default_value_t = 0)]
        max_ips: i32,

        /// Description for the token
        #[arg(short, long)]
        description: Option<String>,
    },

    /// List all existing tokens
    List,

    /// Revoke (delete) a token by ID
    Revoke {
        /// Token ID to revoke
        id: String,
    },
}

fn main() {
    let cli = Cli::parse();

    // Initialize logger
    antigravity_tools_lib::modules::logger::init_logger();

    // Initialize databases
    if let Err(e) = antigravity_tools_lib::modules::token_stats::init_db() {
        tracing::error!("Failed to initialize token stats database: {}", e);
    }
    if let Err(e) = antigravity_tools_lib::modules::security_db::init_db() {
        tracing::error!("Failed to initialize security database: {}", e);
    }
    if let Err(e) = antigravity_tools_lib::modules::user_token_db::init_db() {
        tracing::error!("Failed to initialize user token database: {}", e);
    }

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    match cli.command {
        Commands::Start {
            port,
            accounts,
            lan,
            auto_token,
            log_dir,
        } => {
            rt.block_on(async {
                run_start(port, accounts, lan, auto_token, log_dir).await;
            });
        }
        Commands::Token { action } => {
            rt.block_on(async {
                run_token_command(action).await;
            });
        }
        Commands::Info => {
            run_info();
        }
    }
}

async fn run_start(port: u16, accounts_path: PathBuf, lan: bool, auto_token: bool, log_dir: Option<PathBuf>) {
    println!("══════════════════════════════════════════════════");
    println!("  🚀 Antigravity Ultra — Starting Proxy Server");
    println!("══════════════════════════════════════════════════");

    // Enable proxy request/response file logging
    if let Some(ref dir) = log_dir {
        antigravity_tools_lib::modules::proxy_file_logger::set_custom_log_dir(dir.clone());
        println!("\n📁 Proxy logs directory: {}", dir.display());
    } else {
        let default_dir = dirs::home_dir()
            .map(|h| h.join(".antigravity_tools").join("logs"))
            .unwrap_or_else(|| PathBuf::from("./logs"));
        println!("\n📁 Proxy logs directory: {}", default_dir.display());
    }
    antigravity_tools_lib::modules::proxy_file_logger::set_file_logging_enabled(true);

    // Auto-cleanup old proxy request logs (keep 7 days)
    match antigravity_tools_lib::modules::proxy_file_logger::cleanup_old_proxy_logs(7) {
        Ok(deleted) if deleted > 0 => println!("   🧹 Cleaned up {} old proxy log files", deleted),
        _ => {}
    }

    // 1. Load accounts from file
    if accounts_path.exists() {
        println!("\n📂 Loading accounts from: {}", accounts_path.display());
        match antigravity_tools_lib::modules::cli_accounts::import_all_from_file(&accounts_path)
            .await
        {
            Ok((success, total)) => {
                println!("   ✅ {}/{} accounts loaded\n", success, total);
            }
            Err(e) => {
                eprintln!("   ❌ Failed to load accounts: {}", e);
                eprintln!("   ⚠️  Continuing with existing accounts...\n");
            }
        }
    } else {
        eprintln!(
            "⚠️  Accounts file not found: {}",
            accounts_path.display()
        );
        eprintln!("   Continuing with existing accounts in ~/.antigravity_tools/\n");
    }

    // 2. Load or create proxy config
    let mut config = antigravity_tools_lib::modules::config::load_app_config()
        .unwrap_or_else(|_| antigravity_tools_lib::models::AppConfig::new());

    config.proxy.port = port;
    config.proxy.allow_lan_access = lan;
    config.proxy.enabled = true;
    config.proxy.enable_logging = true;

    // Force auth mode to AllExceptHealth for security
    config.proxy.auth_mode =
        antigravity_tools_lib::proxy::ProxyAuthMode::AllExceptHealth;

    // Save config
    let _ = antigravity_tools_lib::modules::config::save_app_config(&config);

    // 3. Auto-create a User Token if none exist
    if auto_token {
        match antigravity_tools_lib::modules::user_token_db::list_tokens() {
            Ok(tokens) if tokens.is_empty() => {
                println!("🔑 No User Tokens found. Creating default token...");
                match antigravity_tools_lib::modules::user_token_db::create_token(
                    "default".to_string(),
                    "never".to_string(),
                    Some("Auto-generated by antigravity-ultra".to_string()),
                    0,
                    None,
                    None,
                    None,
                ) {
                    Ok(token) => {
                        println!("   ✅ Default token created: {}\n", token.token);
                    }
                    Err(e) => {
                        eprintln!("   ❌ Failed to create default token: {}", e);
                    }
                }
            }
            _ => {}
        }
    }

    // 4. Start the proxy server
    let proxy_state =
        antigravity_tools_lib::commands::proxy::ProxyServiceState::new();
    let cf_state = Arc::new(
        antigravity_tools_lib::commands::cloudflared::CloudflaredState::new(),
    );

    match antigravity_tools_lib::commands::proxy::internal_start_proxy_service(
        config.proxy.clone(),
        &proxy_state,
        antigravity_tools_lib::modules::integration::SystemManager::Headless,
        cf_state,
    )
    .await
    {
        Ok(status) => {
            let bind_addr = if lan { "0.0.0.0" } else { "127.0.0.1" };

            println!("══════════════════════════════════════════════════");
            println!("  ✅ Proxy Server Running");
            println!("══════════════════════════════════════════════════");
            println!("  📍 Address:  http://{}:{}", bind_addr, port);
            println!("  👥 Accounts: {}", status.active_accounts);

            // Show API key info
            println!("  🔑 API Key:  {}", config.proxy.api_key);

            // Show log locations
            let log_dir_display = log_dir.as_ref()
                .map(|d| d.display().to_string())
                .unwrap_or_else(|| {
                    dirs::home_dir()
                        .map(|h| h.join(".antigravity_tools").join("logs").display().to_string())
                        .unwrap_or_else(|| "./logs".to_string())
                });
            println!("  📝 App logs: {}/app.log.*", log_dir_display);
            println!("  📋 Request logs: {}/proxy_requests.log.*", log_dir_display);

            // Show User Tokens
            if let Ok(tokens) = antigravity_tools_lib::modules::user_token_db::list_tokens() {
                if !tokens.is_empty() {
                    println!("\n  📋 User Tokens (for external tools):");
                    for t in &tokens {
                        let status_icon = if t.enabled { "🟢" } else { "🔴" };
                        println!(
                            "     {} {} (user: {}, expires: {})",
                            status_icon, t.token, t.username, t.expires_type
                        );
                    }
                }
            }

            println!("\n  💡 Example usage:");
            println!(
                "     curl http://{}:{}/v1/chat/completions \\",
                bind_addr, port
            );
            println!("       -H \"Authorization: Bearer <YOUR_TOKEN>\" \\");
            println!("       -H \"Content-Type: application/json\" \\");
            println!("       -d '{{\"model\":\"gemini-2.5-flash\",\"messages\":[{{\"role\":\"user\",\"content\":\"Hello\"}}]}}'");
            println!("\n     💡 Also supports: claude-sonnet-4-20250514, claude-sonnet-4-5, claude-opus-4");
            println!("══════════════════════════════════════════════════\n");

            // Wait for Ctrl-C
            tokio::signal::ctrl_c().await.ok();
            println!("\n🛑 Shutting down...");
        }
        Err(e) => {
            eprintln!("❌ Failed to start proxy service: {}", e);
            std::process::exit(1);
        }
    }
}

async fn run_token_command(action: TokenAction) {
    match action {
        TokenAction::Create {
            username,
            expires,
            max_ips,
            description,
        } => {
            match antigravity_tools_lib::modules::user_token_db::create_token(
                username.clone(),
                expires.clone(),
                description,
                max_ips,
                None,
                None,
                None,
            ) {
                Ok(token) => {
                    println!("✅ Token created successfully!");
                    println!("══════════════════════════════════════════════════");
                    println!("  🔑 Token:    {}", token.token);
                    println!("  👤 Username: {}", token.username);
                    println!("  ⏰ Expires:  {}", token.expires_type);
                    println!("  🌐 Max IPs:  {}", if token.max_ips == 0 { "unlimited".to_string() } else { token.max_ips.to_string() });
                    println!("  📝 ID:       {}", token.id);
                    println!("══════════════════════════════════════════════════");
                    println!("\n💡 Use this token as your API key:");
                    println!("   Authorization: Bearer {}", token.token);
                }
                Err(e) => {
                    eprintln!("❌ Failed to create token: {}", e);
                    std::process::exit(1);
                }
            }
        }
        TokenAction::List => {
            match antigravity_tools_lib::modules::user_token_db::list_tokens() {
                Ok(tokens) => {
                    if tokens.is_empty() {
                        println!("📋 No tokens found. Create one with: antigravity-ultra token create --username <name>");
                        return;
                    }

                    println!("📋 User Tokens ({} total):", tokens.len());
                    println!("══════════════════════════════════════════════════");
                    for t in &tokens {
                        let status = if t.enabled { "🟢 Active" } else { "🔴 Disabled" };
                        println!("  {} {}", status, t.token);
                        println!("     User: {} | Expires: {} | Requests: {} | ID: {}",
                            t.username, t.expires_type, t.total_requests, t.id);
                        if let Some(ref desc) = t.description {
                            if !desc.is_empty() {
                                println!("     Desc: {}", desc);
                            }
                        }
                        println!();
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to list tokens: {}", e);
                    std::process::exit(1);
                }
            }
        }
        TokenAction::Revoke { id } => {
            match antigravity_tools_lib::modules::user_token_db::delete_token(&id) {
                Ok(_) => {
                    println!("✅ Token {} revoked successfully", id);
                }
                Err(e) => {
                    eprintln!("❌ Failed to revoke token: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn run_info() {
    // Load config
    let config = antigravity_tools_lib::modules::config::load_app_config()
        .unwrap_or_else(|_| antigravity_tools_lib::models::AppConfig::new());

    let port = config.proxy.port;
    let bind = if config.proxy.allow_lan_access {
        "0.0.0.0"
    } else {
        "127.0.0.1"
    };

    println!("══════════════════════════════════════════════════");
    println!("  🔧 Antigravity Ultra — Connection Info");
    println!("══════════════════════════════════════════════════");
    println!("  📍 Base URL: http://{}:{}", bind, port);
    println!("  🔑 API Key:  {}", config.proxy.api_key);

    // Show endpoints
    println!("\n  📡 API Endpoints:");
    println!("     POST /v1/chat/completions     (OpenAI/Anthropic compatible)");
    println!("     POST /v1/messages             (Anthropic native)");
    println!("     GET  /v1/models               (List available models)");
    println!("     GET  /healthz                  (Health check)");

    // Show User Tokens
    if let Ok(tokens) = antigravity_tools_lib::modules::user_token_db::list_tokens() {
        if !tokens.is_empty() {
            println!("\n  📋 Active User Tokens:");
            for t in tokens.iter().filter(|t| t.enabled) {
                println!("     {} (user: {})", t.token, t.username);
            }
        } else {
            println!("\n  ⚠️  No User Tokens. Create one:");
            println!("     antigravity-ultra token create --username my-tool");
        }
    }

    // Example curl
    let example_token = antigravity_tools_lib::modules::user_token_db::list_tokens()
        .ok()
        .and_then(|t| t.first().map(|t| t.token.clone()))
        .unwrap_or_else(|| config.proxy.api_key.clone());

    println!("\n  💡 Example curl:");
    println!(
        "     curl http://{}:{}/v1/chat/completions \\",
        bind, port
    );
    println!("       -H \"Authorization: Bearer {}\" \\", example_token);
    println!("       -H \"Content-Type: application/json\" \\");
    println!(
        "       -d '{{\"model\":\"gemini-2.5-flash\",\"messages\":[{{\"role\":\"user\",\"content\":\"Hello\"}}]}}'"
    );
    println!("\n     💡 Also supports: claude-sonnet-4-20250514, claude-sonnet-4-5, claude-opus-4");
    println!("══════════════════════════════════════════════════");
}
