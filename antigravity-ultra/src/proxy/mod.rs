mod token_manager;

use crate::account::import::AccountEntry;
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{any, get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use token_manager::TokenManager;
use tower_http::cors::{Any, CorsLayer};

/// Configuration for starting the proxy
pub struct ProxyStartConfig {
    pub port: u16,
    pub lan: bool,
    pub auto_refresh: bool,
    pub healthcheck_interval: u64,
    pub accounts: Vec<AccountEntry>,
}

/// Shared application state
struct AppState {
    token_manager: Arc<TokenManager>,
}

/// Start the proxy server
pub async fn start_proxy_server(config: ProxyStartConfig) -> anyhow::Result<()> {
    // Initialize token manager with loaded accounts
    let token_manager = Arc::new(TokenManager::new(config.accounts, config.auto_refresh).await);

    let account_count = token_manager.account_count();
    tracing::info!(
        "Token pool initialized with {} active accounts",
        account_count
    );

    let state = Arc::new(AppState {
        token_manager: token_manager.clone(),
    });

    // Start healthcheck background task
    if config.healthcheck_interval > 0 && account_count > 0 {
        let tm = token_manager.clone();
        let interval = config.healthcheck_interval;
        tokio::spawn(async move {
            healthcheck_loop(tm, interval).await;
        });
    }

    // Build router
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // OpenAI-compatible endpoints
        .route("/v1/chat/completions", post(handle_chat_completions))
        .route("/v1/models", get(handle_models))
        // Anthropic-compatible endpoint
        .route("/v1/messages", post(handle_chat_completions))
        // Health & status
        .route("/healthz", get(handle_health))
        .route("/health", get(handle_health))
        // Admin API
        .route("/api/status", get(handle_api_status))
        .route("/api/accounts", get(handle_api_accounts))
        .route("/api/quota", get(handle_api_quota))
        .route("/api/tokens", get(handle_api_tokens))
        // Catch-all for unsupported paths
        .fallback(any(handle_fallback))
        .layer(cors)
        .with_state(state);

    let bind_addr: SocketAddr = if config.lan {
        format!("0.0.0.0:{}", config.port).parse()?
    } else {
        format!("127.0.0.1:{}", config.port).parse()?
    };

    tracing::info!("🌐 Proxy server listening on http://{}", bind_addr);

    // Print user tokens for convenience
    if let Ok(tokens) = crate::user_token::list_tokens() {
        if !tokens.is_empty() {
            tracing::info!("🔑 Available API keys:");
            for t in &tokens {
                tracing::info!("   {} (user: {}, expires: {})", t.token, t.username, t.expires_type);
            }
        }
    }

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;

    // Graceful shutdown on Ctrl+C
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        tracing::info!("🛑 Shutting down proxy server...");
    };

    axum::serve(
        listener,
        app.into_make_service(),
    )
    .with_graceful_shutdown(shutdown_signal)
    .await?;

    tracing::info!("Proxy server stopped.");
    Ok(())
}

// ──────────────────────────────────────────────────────────────
// Route handlers
// ──────────────────────────────────────────────────────────────

/// POST /v1/chat/completions or /v1/messages
async fn handle_chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Response {
    let client_ip = "127.0.0.1".to_string();

    // 1. Validate API key
    let api_key = extract_api_key(&headers);
    if let Some(key) = &api_key {
        match crate::user_token::validate_token(key, &client_ip) {
            Ok((true, _)) => {}
            Ok((false, reason)) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": {
                            "message": reason.unwrap_or("Unauthorized".to_string()),
                            "type": "authentication_error"
                        }
                    })),
                )
                    .into_response();
            }
            Err(e) => {
                tracing::error!("Token validation error: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": {"message": "Internal server error"}})),
                )
                    .into_response();
            }
        }
    }

    // 2. Get next available token from pool
    let account = match state.token_manager.get_next_account().await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": {
                        "message": "No available accounts in token pool",
                        "type": "server_error"
                    }
                })),
            )
                .into_response();
        }
    };

    tracing::info!(
        "[Proxy] Routing request from {} to account {}",
        client_ip,
        account.email
    );

    // 3. Determine target URL based on request body
    let body_json: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": {"message": format!("Invalid JSON: {}", e)}})),
            )
                .into_response();
        }
    };

    let _model = body_json
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("gemini-2.0-flash");

    // Build upstream URL
    let project_id = account
        .token
        .project_id
        .as_deref()
        .unwrap_or("bamboo-precept-lgxtn");

    let upstream_url = format!(
        "https://cloudcode-pa.googleapis.com/v1/projects/{}/locations/global/codeAssistChats/-:generateContent",
        project_id
    );

    let is_stream = body_json
        .get("stream")
        .and_then(|s| s.as_bool())
        .unwrap_or(false);

    // 4. Forward request to upstream
    let http_client = crate::utils::http::get_streaming_client();

    let upstream_response = http_client
        .post(&upstream_url)
        .header("Authorization", format!("Bearer {}", account.token.access_token))
        .header("Content-Type", "application/json")
        .header("User-Agent", crate::constants::USER_AGENT.as_str())
        .body(body.clone())
        .send()
        .await;

    match upstream_response {
        Ok(resp) => {
            let status = resp.status();

            if is_stream {
                // Stream response back
                let stream = resp.bytes_stream();
                let body = Body::from_stream(stream);

                Response::builder()
                    .status(status.as_u16())
                    .header("content-type", "text/event-stream")
                    .header("cache-control", "no-cache")
                    .header("connection", "keep-alive")
                    .body(body)
                    .unwrap_or_else(|_| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Stream error".to_string(),
                        )
                            .into_response()
                    })
            } else {
                let resp_status = status.as_u16();
                let resp_body = resp.text().await.unwrap_or_default();

                Response::builder()
                    .status(resp_status)
                    .header("content-type", "application/json")
                    .body(Body::from(resp_body))
                    .unwrap_or_else(|_| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Response error".to_string(),
                        )
                            .into_response()
                    })
            }
        }
        Err(e) => {
            tracing::error!("[Proxy] Upstream error: {}", e);
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "error": {
                        "message": format!("Upstream error: {}", e),
                        "type": "server_error"
                    }
                })),
            )
                .into_response()
        }
    }
}

/// GET /v1/models
async fn handle_models(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let models = state.token_manager.get_available_models();

    let model_list: Vec<Value> = models
        .iter()
        .map(|m| {
            json!({
                "id": m,
                "object": "model",
                "created": 1700000000,
                "owned_by": "google"
            })
        })
        .collect();

    Json(json!({
        "object": "list",
        "data": model_list
    }))
}

/// GET /healthz
async fn handle_health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let account_count = state.token_manager.account_count();
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "accounts": account_count,
    }))
}

/// GET /api/status
async fn handle_api_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let account_count = state.token_manager.account_count();
    let tokens = crate::user_token::list_tokens().unwrap_or_default();

    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running",
        "accounts_loaded": account_count,
        "user_tokens": tokens.len(),
        "session_id": crate::constants::SESSION_ID.as_str(),
    }))
}

/// GET /api/accounts
async fn handle_api_accounts(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let accounts = state.token_manager.get_account_summaries();
    Json(json!({ "accounts": accounts }))
}

/// GET /api/quota
async fn handle_api_quota(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let quotas = state.token_manager.get_all_quotas();
    Json(json!({ "quotas": quotas }))
}

/// GET /api/tokens
async fn handle_api_tokens() -> impl IntoResponse {
    let tokens = crate::user_token::list_tokens().unwrap_or_default();
    Json(json!({ "tokens": tokens }))
}

/// Fallback handler
async fn handle_fallback(method: Method, uri: axum::http::Uri) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": {
                "message": format!("Endpoint not found: {} {}", method, uri),
                "type": "not_found"
            }
        })),
    )
}

// ──────────────────────────────────────────────────────────────
// Helper functions
// ──────────────────────────────────────────────────────────────

fn extract_api_key(headers: &HeaderMap) -> Option<String> {
    // Check Authorization: Bearer sk-xxx
    if let Some(auth) = headers.get("authorization") {
        if let Ok(value) = auth.to_str() {
            if let Some(token) = value.strip_prefix("Bearer ") {
                if token.starts_with("sk-") {
                    return Some(token.to_string());
                }
            }
        }
    }

    // Check x-api-key header
    if let Some(key) = headers.get("x-api-key") {
        if let Ok(value) = key.to_str() {
            if value.starts_with("sk-") {
                return Some(value.to_string());
            }
        }
    }

    None
}

/// Background healthcheck loop
async fn healthcheck_loop(token_manager: Arc<TokenManager>, interval_seconds: u64) {
    tracing::info!(
        "🏥 Healthcheck started (interval: {}s)",
        interval_seconds
    );

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(interval_seconds)).await;
        tracing::info!("[Healthcheck] Running periodic check...");
        token_manager.refresh_all_tokens().await;
    }
}
