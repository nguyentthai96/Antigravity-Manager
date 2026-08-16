mod anthropic_bridge;
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

/// Upstream v1internal base URLs (fallback order: Sandbox → Daily → Prod)
const V1_INTERNAL_URLS: [&str; 3] = [
    "https://daily-cloudcode-pa.sandbox.googleapis.com/v1internal",
    "https://daily-cloudcode-pa.googleapis.com/v1internal",
    "https://cloudcode-pa.googleapis.com/v1internal",
];

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

    // 3. Parse request body
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

    let raw_model = body_json
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("gemini-2.0-flash");

    // Map model name to v1internal-compatible name (matches Tauri CLAUDE_TO_GEMINI table)
    let model = map_model_name(raw_model);

    // 4. Detect protocol format (Anthropic vs OpenAI)
    let is_anthropic = anthropic_bridge::is_anthropic_format(&body_json, &headers);
    let is_stream = if is_anthropic {
        anthropic_bridge::detect_stream(&body_json, &headers)
    } else {
        body_json
            .get("stream")
            .and_then(|s| s.as_bool())
            .unwrap_or(false)
    };

    let is_claude = model.to_lowercase().contains("claude");

    tracing::info!(
        "[Proxy] Protocol: {}, Stream: {}, Model: {} (raw: {}), Claude: {}",
        if is_anthropic { "Anthropic" } else { "OpenAI" },
        is_stream,
        model,
        raw_model,
        is_claude,
    );

    // 5. Transform request body to Gemini v1internal format
    let gemini_body = if is_anthropic {
        anthropic_bridge::transform_anthropic_to_gemini(&body_json, &model, &account)
    } else {
        transform_openai_to_gemini(&body_json, &model, &account)
    };

    // 6. Build upstream URL
    let (method_name, query_string) = if is_stream {
        ("streamGenerateContent", Some("alt=sse"))
    } else {
        ("generateContent", None)
    };

    let http_client = crate::utils::http::get_streaming_client();
    let body_str = serde_json::to_string(&gemini_body).unwrap_or_default();

    // 7. Try endpoints with fallback (Sandbox → Daily → Prod)
    // Two-pass strategy (matches Tauri source):
    //   Pass 0: WITH x-goog-user-project header
    //   Pass 1: WITHOUT header (triggered only if 403 SERVICE_DISABLED detected)
    let mut skip_project_header = false;

    for pass in 0..2u8 {
        for base_url in &V1_INTERNAL_URLS {
            let upstream_url = if let Some(qs) = query_string {
                format!("{}:{}?{}", base_url, method_name, qs)
            } else {
                format!("{}:{}", base_url, method_name)
            };

            let mut req = http_client
                .post(&upstream_url)
                .header("Authorization", format!("Bearer {}", account.token.access_token))
                .header("Content-Type", "application/json")
                .header("User-Agent", crate::constants::USER_AGENT.as_str())
                .header("x-client-name", "antigravity")
                .header("x-client-version", crate::constants::CURRENT_VERSION.as_str())
                .header("x-vscode-sessionid", crate::constants::SESSION_ID.as_str());

            // Inject project header (skip on pass 1 after 403 SERVICE_DISABLED)
            if !skip_project_header {
                if let Some(ref project_id) = account.token.project_id {
                    req = req.header("x-goog-user-project", project_id.as_str());
                }
            }

            // Claude-specific headers
            if is_claude {
                req = req.header("anthropic-beta", "claude-code-20250219");
            }

            let upstream_response = req.body(body_str.clone()).send().await;

            match upstream_response {
                Ok(resp) => {
                    let status = resp.status();

                    // Handle 403 → on first pass, trigger retry without project header
                    if status.as_u16() == 403 && !skip_project_header {
                        let err_body = resp.text().await.unwrap_or_default();
                        if err_body.contains("SERVICE_DISABLED") || err_body.contains("has not been used") {
                            tracing::warn!(
                                "[Proxy] 403 SERVICE_DISABLED — will retry without x-goog-user-project (account: {})",
                                account.email
                            );
                            skip_project_header = true;
                            break; // Break inner loop, trigger pass 1
                        }
                        // Other 403 errors — try next endpoint
                        tracing::warn!("[Proxy] Endpoint {} returned 403 — trying next", upstream_url);
                        continue;
                    }

                    // Handle 403 on pass 2 (already without project header) — try next endpoint
                    if status.as_u16() == 403 {
                        tracing::warn!("[Proxy] Endpoint {} returned 403 (no project header) — trying next", upstream_url);
                        continue;
                    }

                    // If 404 or server error, try next endpoint
                    if status.as_u16() == 404 || status.is_server_error() {
                        let err_body = resp.text().await.unwrap_or_default();
                        tracing::warn!(
                            "[Proxy] Endpoint {} returned {} — trying next fallback",
                            upstream_url, status
                        );
                        continue;
                    }

                    // ─── SUCCESS: Build response based on protocol ───
                    if is_stream {
                        if is_anthropic {
                            tracing::info!(
                                "[Proxy] Streaming Gemini→Anthropic SSE bridge for {} ({})",
                                model, upstream_url,
                            );
                            let body = anthropic_bridge::create_anthropic_sse_stream(resp, raw_model);
                            return Response::builder()
                                .status(200)
                                .header("content-type", "text/event-stream")
                                .header("cache-control", "no-cache")
                                .header("connection", "keep-alive")
                                .body(body)
                                .unwrap_or_else(|_| {
                                    (StatusCode::INTERNAL_SERVER_ERROR, "Stream error".to_string()).into_response()
                                });
                        } else {
                            let stream = resp.bytes_stream();
                            let body = Body::from_stream(stream);
                            return Response::builder()
                                .status(status.as_u16())
                                .header("content-type", "text/event-stream")
                                .header("cache-control", "no-cache")
                                .header("connection", "keep-alive")
                                .body(body)
                                .unwrap_or_else(|_| {
                                    (StatusCode::INTERNAL_SERVER_ERROR, "Stream error".to_string()).into_response()
                                });
                        }
                    } else {
                        let resp_status = status.as_u16();
                        let resp_body = resp.text().await.unwrap_or_default();

                        if is_anthropic {
                            tracing::debug!(
                                "[Proxy] Raw upstream body for Anthropic transform ({}B): {}",
                                resp_body.len(),
                                &resp_body[..resp_body.len().min(500)]
                            );
                            let anthropic_resp = anthropic_bridge::create_anthropic_response(&resp_body, raw_model);
                            return Response::builder()
                                .status(200)
                                .header("content-type", "application/json")
                                .body(Body::from(serde_json::to_string(&anthropic_resp).unwrap_or_default()))
                                .unwrap_or_else(|_| {
                                    (StatusCode::INTERNAL_SERVER_ERROR, "Response error".to_string()).into_response()
                                });
                        } else {
                            return Response::builder()
                                .status(resp_status)
                                .header("content-type", "application/json")
                                .body(Body::from(resp_body))
                                .unwrap_or_else(|_| {
                                    (StatusCode::INTERNAL_SERVER_ERROR, "Response error".to_string()).into_response()
                                });
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("[Proxy] Endpoint {} connection error: {} — trying next", upstream_url, e);
                    continue;
                }
            }
        }

        // If pass 0 triggered 403 fallback, continue to pass 1
        if skip_project_header && pass == 0 {
            tracing::info!("[Proxy] Retrying all endpoints without project header...");
            continue;
        }
        break;
    }

    // All endpoints exhausted
    tracing::error!("[Proxy] All upstream endpoints failed");

    if is_anthropic {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "type": "error",
                "error": {
                    "type": "api_error",
                    "message": "All upstream endpoints failed"
                }
            })),
        )
            .into_response()
    } else {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "error": {
                    "message": "All upstream endpoints failed",
                    "type": "server_error"
                }
            })),
        )
            .into_response()
    }
}

/// Transform OpenAI chat completion request to Gemini v1internal format
fn transform_openai_to_gemini(openai_body: &Value, model: &str, account: &crate::models::Account) -> Value {
    let project_id = account
        .token
        .project_id
        .as_deref()
        .unwrap_or("");

    // Convert messages to Gemini contents format
    let mut contents: Vec<Value> = Vec::new();
    let mut system_parts: Vec<Value> = Vec::new();

    if let Some(messages) = openai_body.get("messages").and_then(|m| m.as_array()) {
        for msg in messages {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let content = msg.get("content").and_then(|c| c.as_str()).unwrap_or("");

            match role {
                "system" => {
                    system_parts.push(json!({"text": content}));
                }
                "assistant" | "model" => {
                    contents.push(json!({
                        "role": "model",
                        "parts": [{"text": content}]
                    }));
                }
                _ => {
                    // "user" and anything else
                    contents.push(json!({
                        "role": "user",
                        "parts": [{"text": content}]
                    }));
                }
            }
        }
    }

    // Build generation config
    let mut generation_config = json!({});
    if let Some(temp) = openai_body.get("temperature") {
        generation_config["temperature"] = temp.clone();
    }
    if let Some(max_tokens) = openai_body.get("max_tokens") {
        generation_config["maxOutputTokens"] = max_tokens.clone();
    }
    if let Some(top_p) = openai_body.get("top_p") {
        generation_config["topP"] = top_p.clone();
    }

    // Handle thinking config
    if let Some(thinking) = openai_body.get("thinking") {
        if let Some(budget) = thinking.get("budget_tokens").and_then(|b| b.as_i64()) {
            generation_config["thinkingConfig"] = json!({
                "thinkingBudget": budget
            });
        }
    }

    // Session ID (FNV-1a hash of account ID)
    let session_id = format!("{:x}", {
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in account.id.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    });

    // Build the inner "request" object with stable field ordering (matches Tauri source)
    // Order: systemInstruction → tools → toolConfig → generationConfig → safetySettings → sessionId → contents
    let mut inner_request = serde_json::Map::new();

    // systemInstruction first (stable, benefits prefix caching)
    if !system_parts.is_empty() {
        inner_request.insert("systemInstruction".to_string(), json!({
            "role": "user",
            "parts": system_parts
        }));
    }

    // generationConfig
    inner_request.insert("generationConfig".to_string(), generation_config);

    // safetySettings
    inner_request.insert("safetySettings".to_string(), json!([
        { "category": "HARM_CATEGORY_HARASSMENT", "threshold": "OFF" },
        { "category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "OFF" },
        { "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "OFF" },
        { "category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "OFF" },
    ]));

    // sessionId
    inner_request.insert("sessionId".to_string(), json!(session_id));

    // contents last (dynamic)
    inner_request.insert("contents".to_string(), json!(contents));

    // Build official requestId: agent/{timestamp_ms}/{random_hex_8bytes}
    let timestamp_ms = chrono::Utc::now().timestamp_millis();
    let random_hex = &uuid::Uuid::new_v4().simple().to_string()[..8];
    let official_request_id = format!("agent/{}/{}", timestamp_ms, random_hex);

    // Determine userAgent based on account type
    let is_enterprise = !account.email.ends_with("@gmail.com") && !account.email.ends_with("@googlemail.com");
    let user_agent = if is_enterprise { "jetski" } else { "antigravity" };

    // Build final body with enabledCreditTypes (critical for upstream API)
    let final_body = json!({
        "project": project_id,
        "request": Value::Object(inner_request),
        "model": model,
        "userAgent": user_agent,
        "requestType": "agent",
        "requestId": official_request_id,
        "enabledCreditTypes": ["GOOGLE_ONE_AI"],
    });

    final_body
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

/// Map model names to v1internal-compatible names
/// Matches the Tauri source's CLAUDE_TO_GEMINI mapping table
fn map_model_name(input: &str) -> String {
    match input {
        // Anthropic SDK versioned IDs → internal names
        "claude-sonnet-4-20250514" => "claude-sonnet-4-6".to_string(),
        "claude-sonnet-4-5-20250514" => "claude-sonnet-4-6".to_string(),
        "claude-sonnet-4-5-20250929" => "claude-sonnet-4-6-thinking".to_string(),
        "claude-3-5-sonnet-20241022" => "claude-sonnet-4-6".to_string(),
        "claude-3-5-sonnet-20240620" => "claude-sonnet-4-6".to_string(),

        // Sonnet aliases
        "claude-sonnet-4-5" => "claude-sonnet-4-6".to_string(),
        "claude-sonnet-4-5-thinking" => "claude-sonnet-4-6-thinking".to_string(),
        "claude-sonnet-4-6" => "claude-sonnet-4-6".to_string(),
        "claude-sonnet-4-6-thinking" => "claude-sonnet-4-6-thinking".to_string(),

        // Opus aliases
        "claude-opus-4" => "claude-opus-4-6-thinking".to_string(),
        "claude-opus-4-5-thinking" => "claude-opus-4-6-thinking".to_string(),
        "claude-opus-4-5-20251101" => "claude-opus-4-6-thinking".to_string(),
        "claude-opus-4-6" => "claude-opus-4-6-thinking".to_string(),
        "claude-opus-4-6-thinking" => "claude-opus-4-6-thinking".to_string(),
        "claude-opus-4-6-20260201" => "claude-opus-4-6-thinking".to_string(),

        // Haiku → Sonnet
        "claude-haiku-4" => "claude-sonnet-4-6".to_string(),
        "claude-3-haiku-20240307" => "claude-sonnet-4-6".to_string(),
        "claude-haiku-4-5-20251001" => "claude-sonnet-4-6".to_string(),

        // OpenAI → Gemini
        "gpt-4" | "gpt-4-turbo" | "gpt-4o" | "gpt-4o-mini" => "gemini-2.5-flash".to_string(),
        "gpt-3.5-turbo" | "gpt-3.5-turbo-16k" => "gemini-2.5-flash".to_string(),

        // Gemini pass-through & aliases
        "gemini-3.1-pro-high" => "gemini-pro-agent".to_string(),
        "gemini-3.1-pro" => "gemini-3.1-pro-preview".to_string(),
        "gemini-3-pro-high" => "gemini-pro-agent".to_string(),
        "gemini-3-pro" => "gemini-3-pro-preview".to_string(),

        // Default: pass through unchanged
        _ => input.to_string(),
    }
}

