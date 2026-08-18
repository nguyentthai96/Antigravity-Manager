//! Anthropic ↔ Gemini protocol bridge.
//!
//! Transforms Anthropic Messages API requests to Gemini v1internal format,
//! and Gemini SSE responses back to Anthropic SSE format.

use bytes::Bytes;
use futures::stream::{self, Stream, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::pin::Pin;

// ──────────────────────────────────────────────────────────────
// Request detection
// ──────────────────────────────────────────────────────────────

/// Detect if the incoming request body is in Anthropic Messages API format.
///
/// Heuristics:
///   - Has top-level "system" field (string or array) — Anthropic puts system at top level
///   - Has "tools" with "input_schema" — Anthropic tool format
///   - Does NOT have "messages[].content" as string only — Anthropic supports array content blocks
pub fn is_anthropic_format(body: &Value, headers: &axum::http::HeaderMap) -> bool {
    // Check headers first (fastest)

    // Standard Anthropic SDK header
    if headers.get("anthropic-version").is_some() {
        return true;
    }

    // Anthropic SDK sends x-api-key instead of Authorization: Bearer
    if headers.get("x-api-key").is_some() && !headers.contains_key("authorization") {
        return true;
    }

    // Stainless SDK headers
    if headers.get("x-stainless-stream-helper").is_some() {
        return true;
    }
    if headers.get("x-stainless-helper-method").is_some() {
        return true;
    }

    // Check body structure
    // Anthropic has top-level "system" (not inside messages)
    if body.get("system").is_some() {
        return true;
    }

    // Anthropic uses "max_tokens" at top level (required field)
    // combined with "messages" but no "response_format" (OpenAI-specific)
    if body.get("max_tokens").is_some()
        && body.get("messages").is_some()
        && body.get("response_format").is_none()
    {
        // Additional check: Anthropic message content is array of blocks or string
        // while OpenAI message content is always string
        if let Some(msgs) = body.get("messages").and_then(|m| m.as_array()) {
            if let Some(first) = msgs.first() {
                // Anthropic uses "content" as array of blocks
                if first.get("content").and_then(|c| c.as_array()).is_some() {
                    return true;
                }
            }
        }
    }

    // Anthropic tools use "input_schema", OpenAI uses "parameters"
    if let Some(tools) = body.get("tools").and_then(|t| t.as_array()) {
        if let Some(first) = tools.first() {
            if first.get("input_schema").is_some() {
                return true;
            }
        }
    }

    false
}

/// Detect if the request wants streaming.
///
/// Anthropic SDK uses `client.messages.stream()` which:
///   - Sets `x-stainless-stream-helper: messages` header
///   - Sets `"stream": true` in body
///
/// We check both body field and headers.
pub fn detect_stream(body: &Value, headers: &axum::http::HeaderMap) -> bool {
    // Check body "stream" field
    if let Some(stream) = body.get("stream").and_then(|s| s.as_bool()) {
        return stream;
    }

    // Check Anthropic SDK streaming headers
    if headers.get("x-stainless-stream-helper").is_some() {
        return true;
    }

    false
}

// ──────────────────────────────────────────────────────────────
// Request transformation: Anthropic → Gemini v1internal
// ──────────────────────────────────────────────────────────────

/// Transform an Anthropic Messages API request body into Gemini v1internal format.
pub fn transform_anthropic_to_gemini(
    body: &Value,
    model: &str,
    account: &crate::models::Account,
) -> Value {
    let project_id = account.token.project_id.as_deref().unwrap_or("");

    // 1. First pass: build tool_use_id → function_name map from all messages.
    //    Gemini requires functionResponse.name to be the actual function name,
    //    not the Anthropic tool_use_id. We scan assistant messages for tool_use
    //    blocks to build this mapping before converting any messages.
    let mut tool_id_to_name: HashMap<String, String> = HashMap::new();
    if let Some(messages) = body.get("messages").and_then(|m| m.as_array()) {
        for msg in messages {
            if msg.get("role").and_then(|r| r.as_str()) == Some("assistant") {
                if let Some(Value::Array(blocks)) = msg.get("content") {
                    for block in blocks {
                        if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                            if let (Some(id), Some(name)) = (
                                block.get("id").and_then(|i| i.as_str()),
                                block.get("name").and_then(|n| n.as_str()),
                            ) {
                                tool_id_to_name.insert(id.to_string(), name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    if !tool_id_to_name.is_empty() {
        tracing::info!("[AnthropicBridge] Built tool_id→name map: {:?}", tool_id_to_name);
    }

    // 2. Convert messages to Gemini contents
    let mut contents: Vec<Value> = Vec::new();

    if let Some(messages) = body.get("messages").and_then(|m| m.as_array()) {
        for msg in messages {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let gemini_role = match role {
                "assistant" => "model",
                _ => "user",
            };

            let parts = extract_parts_from_anthropic_message(msg, &tool_id_to_name);
            if !parts.is_empty() {
                contents.push(json!({
                    "role": gemini_role,
                    "parts": parts
                }));
            }
        }
    }

    // 2. Extract system instruction (Anthropic puts it at top level)
    let system_instruction = extract_system_instruction(body);

    // 3. Build generation config
    let mut generation_config = json!({});
    if let Some(max_tokens) = body.get("max_tokens") {
        generation_config["maxOutputTokens"] = max_tokens.clone();
    }
    if let Some(temp) = body.get("temperature") {
        generation_config["temperature"] = temp.clone();
    }
    if let Some(top_p) = body.get("top_p") {
        generation_config["topP"] = top_p.clone();
    }

    // Handle thinking config (Anthropic thinking models)
    if let Some(thinking) = body.get("thinking") {
        if let Some(budget) = thinking.get("budget_tokens").and_then(|b| b.as_i64()) {
            generation_config["thinkingConfig"] = json!({
                "thinkingBudget": budget
            });
            // Ensure maxOutputTokens > thinkingBudget (Gemini requirement)
            let current_max = generation_config.get("maxOutputTokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            if current_max <= budget {
                generation_config["maxOutputTokens"] = json!(budget + 8000);
            }
        }
    }

    // 4. Convert Anthropic tools to Gemini function declarations
    let tools_declarations = convert_anthropic_tools(body);

    // 5. Session ID
    let session_id = format!("{:x}", {
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in account.id.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    });

    // 6. Build inner request with stable field ordering (matches Tauri source)
    // Order: systemInstruction → tools → toolConfig → generationConfig → safetySettings → sessionId → contents
    let mut inner_request = serde_json::Map::new();

    // systemInstruction first (most stable, benefits prefix caching)
    if let Some(system) = system_instruction {
        inner_request.insert("systemInstruction".to_string(), system);
    }

    // tools
    if let Some(tools) = tools_declarations {
        inner_request.insert("tools".to_string(), tools);
        // Inject toolConfig when tools are present (match Tauri: VALIDATED mode)
        inner_request.insert("toolConfig".to_string(), json!({
            "functionCallingConfig": { "mode": "VALIDATED" }
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

    // contents last (dynamic, changes every turn)
    inner_request.insert("contents".to_string(), json!(contents));

    // 7. Build official requestId format: agent/{timestamp_ms}/{random_hex_8bytes}
    let timestamp_ms = chrono::Utc::now().timestamp_millis();
    let random_hex = &uuid::Uuid::new_v4().simple().to_string()[..8];
    let official_request_id = format!("agent/{}/{}", timestamp_ms, random_hex);

    // Determine userAgent based on account type
    let is_enterprise = !account.email.ends_with("@gmail.com") && !account.email.ends_with("@googlemail.com");
    let user_agent = if is_enterprise { "jetski" } else { "antigravity" };

    // 8. Final wrapper with enabledCreditTypes (critical for upstream API)
    let mut final_body = json!({
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

/// Extract parts from an Anthropic message.
///
/// Anthropic content can be:
///   - A string: `"content": "Hello"` (simple text)
///   - An array of content blocks: `"content": [{"type": "text", "text": "..."}, {"type": "tool_use", ...}]`
///   - An array with tool_result blocks (from user role): `[{"type": "tool_result", ...}]`
fn extract_parts_from_anthropic_message(msg: &Value, tool_id_to_name: &HashMap<String, String>) -> Vec<Value> {
    let mut parts = Vec::new();

    match msg.get("content") {
        Some(Value::String(text)) => {
            parts.push(json!({"text": text}));
        }
        Some(Value::Array(blocks)) => {
            for block in blocks {
                let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                match block_type {
                    "text" => {
                        if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                            // Skip empty text blocks — they cause upstream errors
                            // (e.g. "messages.1.content.2.text.text: Field required")
                            // because cloudcode can't reconstruct them properly.
                            if !text.is_empty() {
                                parts.push(json!({"text": text}));
                            }
                        }
                    }
                    "thinking" => {
                        // Thinking blocks from previous assistant turns — include as text
                        if let Some(thinking_text) = block.get("thinking").and_then(|t| t.as_str())
                        {
                            parts.push(json!({"thought": true, "text": thinking_text}));
                        }
                    }
                    "tool_use" => {
                        // Assistant requesting tool use
                        let name = block.get("name").and_then(|n| n.as_str()).unwrap_or("");
                        let id = block.get("id").and_then(|i| i.as_str()).unwrap_or("");
                        let input = block.get("input").cloned().unwrap_or(json!({}));
                        // Include "id" so upstream cloudcode can reconstruct the
                        // Anthropic tool_use.id field (required for multi-turn).
                        let mut fc = json!({
                            "functionCall": {
                                "name": name,
                                "args": input
                            }
                        });
                        if !id.is_empty() {
                            fc["functionCall"]["id"] = json!(id);
                        }
                        parts.push(fc);
                    }
                    "tool_result" => {
                        // User providing tool results
                        let tool_use_id =
                            block.get("tool_use_id").and_then(|t| t.as_str()).unwrap_or("");
                        // CRITICAL: Gemini requires functionResponse.name to be the
                        // actual function name (e.g. "list_dir"), NOT the Anthropic
                        // tool_use_id (e.g. "toolu_xxxx"). Look up from the map we built.
                        let function_name = tool_id_to_name
                            .get(tool_use_id)
                            .map(|s| s.as_str())
                            .unwrap_or_else(|| {
                                tracing::warn!(
                                    "[AnthropicBridge] tool_use_id '{}' not found in tool_id→name map, using as-is",
                                    tool_use_id
                                );
                                tool_use_id
                            });
                        let content = match block.get("content") {
                            Some(Value::String(s)) => s.clone(),
                            Some(Value::Array(arr)) => {
                                // Extract text from content array
                                arr.iter()
                                    .filter_map(|b| {
                                        if b.get("type").and_then(|t| t.as_str()) == Some("text") {
                                            b.get("text").and_then(|t| t.as_str()).map(String::from)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            }
                            _ => String::new(),
                        };
                        let is_error = block
                            .get("is_error")
                            .and_then(|e| e.as_bool())
                            .unwrap_or(false);
                        tracing::debug!(
                            "[AnthropicBridge] tool_result: id='{}' → name='{}', content_len={}, is_error={}",
                            tool_use_id, function_name, content.len(), is_error
                        );
                        parts.push(json!({
                            "functionResponse": {
                                "name": function_name,
                                "id": tool_use_id,
                                "response": {
                                    "result": content,
                                    "error": is_error
                                }
                            }
                        }));
                    }
                    "image" => {
                        // Image content — pass through as inline data
                        if let Some(source) = block.get("source") {
                            let media_type = source
                                .get("media_type")
                                .and_then(|m| m.as_str())
                                .unwrap_or("image/png");
                            let data = source.get("data").and_then(|d| d.as_str()).unwrap_or("");
                            parts.push(json!({
                                "inlineData": {
                                    "mimeType": media_type,
                                    "data": data
                                }
                            }));
                        }
                    }
                    _ => {
                        // Unknown block type — include as text if possible
                        if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                            parts.push(json!({"text": text}));
                        }
                    }
                }
            }
        }
        _ => {}
    }

    parts
}

/// Extract system instruction from Anthropic body.
/// Anthropic "system" can be a string or an array of content blocks.
fn extract_system_instruction(body: &Value) -> Option<Value> {
    match body.get("system") {
        Some(Value::String(text)) => Some(json!({
            "role": "user",
            "parts": [{"text": text}]
        })),
        Some(Value::Array(blocks)) => {
            let parts: Vec<Value> = blocks
                .iter()
                .filter_map(|b| {
                    if b.get("type").and_then(|t| t.as_str()) == Some("text") {
                        b.get("text")
                            .and_then(|t| t.as_str())
                            .map(|text| json!({"text": text}))
                    } else {
                        None
                    }
                })
                .collect();
            if parts.is_empty() {
                None
            } else {
                Some(json!({
                    "role": "user",
                    "parts": parts
                }))
            }
        }
        _ => None,
    }
}

/// Fields that Gemini does not support in JSON Schema for function declarations.
/// These must be stripped recursively from tool parameter definitions.
const UNSUPPORTED_SCHEMA_FIELDS: &[&str] = &[
    "$schema", "$id", "$ref", "$comment",
    "propertyNames", "const", "exclusiveMinimum", "exclusiveMaximum",
    "if", "then", "else", "allOf", "anyOf", "oneOf", "not",
    "patternProperties", "additionalItems", "contains",
    "dependencies", "contentMediaType", "contentEncoding",
    "examples", "default", "readOnly", "writeOnly",
    "minContains", "maxContains", "deprecated",
    "$defs", "definitions",
];

/// Recursively strip unsupported JSON Schema fields from a value.
fn strip_unsupported_schema_fields(value: &mut Value) {
    match value {
        Value::Object(map) => {
            // Remove unsupported top-level fields
            for field in UNSUPPORTED_SCHEMA_FIELDS {
                map.remove(*field);
            }
            // Also handle "any_of" which Gemini doesn't support
            // Recurse into remaining fields
            for (_, v) in map.iter_mut() {
                strip_unsupported_schema_fields(v);
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                strip_unsupported_schema_fields(item);
            }
        }
        _ => {}
    }
}

/// Convert Anthropic tools format to Gemini function declarations.
///
/// Anthropic format:
/// ```json
/// {"name": "grep_search", "description": "...", "input_schema": {"type": "object", "properties": {...}}}
/// ```
///
/// Gemini format:
/// ```json
/// [{"functionDeclarations": [{"name": "grep_search", "description": "...", "parameters": {...}}]}]
/// ```
fn convert_anthropic_tools(body: &Value) -> Option<Value> {
    let tools = body.get("tools")?.as_array()?;
    if tools.is_empty() {
        return None;
    }

    let declarations: Vec<Value> = tools
        .iter()
        .filter_map(|tool| {
            let name = tool.get("name")?.as_str()?;
            let description = tool
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");
            let mut parameters = tool
                .get("input_schema")
                .cloned()
                .unwrap_or(json!({"type": "object", "properties": {}}));

            // Strip unsupported JSON Schema fields for Gemini compatibility
            strip_unsupported_schema_fields(&mut parameters);

            Some(json!({
                "name": name,
                "description": description,
                "parameters": parameters
            }))
        })
        .collect();

    if declarations.is_empty() {
        None
    } else {
        Some(json!([{
            "functionDeclarations": declarations
        }]))
    }
}

// ──────────────────────────────────────────────────────────────
// Response transformation: Gemini SSE → Anthropic SSE
// ──────────────────────────────────────────────────────────────

/// State machine for converting Gemini SSE stream to Anthropic SSE stream.
struct AnthropicSseConverter {
    model: String,
    message_id: String,
    content_index: usize,
    started: bool,
    total_input_tokens: u64,
    total_output_tokens: u64,
    has_tool_use: bool,
    /// Buffer for incomplete SSE lines
    line_buffer: String,
    /// Track the currently open block type: None, "text", "thinking", "tool_use"
    current_block_type: Option<String>,
}

impl AnthropicSseConverter {
    fn new(model: &str) -> Self {
        let message_id = format!("msg_{}", uuid::Uuid::new_v4().to_string().replace('-', "")[..24].to_string());
        Self {
            model: model.to_string(),
            message_id,
            content_index: 0,
            started: false,
            total_input_tokens: 0,
            total_output_tokens: 0,
            has_tool_use: false,
            line_buffer: String::new(),
            current_block_type: None,
        }
    }

    /// Generate the initial message_start event
    fn message_start(&mut self, input_tokens: u64) -> String {
        self.started = true;
        self.total_input_tokens = input_tokens;
        format!(
            "event: message_start\ndata: {}\n\n",
            json!({
                "type": "message_start",
                "message": {
                    "id": self.message_id,
                    "type": "message",
                    "role": "assistant",
                    "content": [],
                    "model": self.model,
                    "stop_reason": null,
                    "stop_sequence": null,
                    "usage": {
                        "input_tokens": input_tokens,
                        "output_tokens": 1
                    }
                }
            })
        )
    }

    /// Generate content_block_start for a text block
    fn text_block_start(&mut self) -> String {
        let idx = self.content_index;
        format!(
            "event: content_block_start\ndata: {}\n\n",
            json!({
                "type": "content_block_start",
                "index": idx,
                "content_block": {
                    "type": "text",
                    "text": ""
                }
            })
        )
    }

    /// Generate content_block_start for a thinking block
    fn thinking_block_start(&mut self) -> String {
        let idx = self.content_index;
        format!(
            "event: content_block_start\ndata: {}\n\n",
            json!({
                "type": "content_block_start",
                "index": idx,
                "content_block": {
                    "type": "thinking",
                    "thinking": ""
                }
            })
        )
    }

    /// Generate a text delta
    fn text_delta(&self, text: &str) -> String {
        format!(
            "event: content_block_delta\ndata: {}\n\n",
            json!({
                "type": "content_block_delta",
                "index": self.content_index,
                "delta": {
                    "type": "text_delta",
                    "text": text
                }
            })
        )
    }

    /// Generate a thinking delta
    fn thinking_delta(&self, text: &str) -> String {
        format!(
            "event: content_block_delta\ndata: {}\n\n",
            json!({
                "type": "content_block_delta",
                "index": self.content_index,
                "delta": {
                    "type": "thinking_delta",
                    "thinking": text
                }
            })
        )
    }

    /// Generate content_block_stop
    fn content_block_stop(&mut self) -> String {
        let idx = self.content_index;
        self.content_index += 1;
        format!(
            "event: content_block_stop\ndata: {}\n\n",
            json!({
                "type": "content_block_stop",
                "index": idx
            })
        )
    }

    /// Generate tool_use content block start
    fn tool_use_block_start(&mut self, id: &str, name: &str) -> String {
        self.has_tool_use = true;
        let idx = self.content_index;
        format!(
            "event: content_block_start\ndata: {}\n\n",
            json!({
                "type": "content_block_start",
                "index": idx,
                "content_block": {
                    "type": "tool_use",
                    "id": id,
                    "name": name,
                    "input": {}
                }
            })
        )
    }

    /// Generate tool_use input JSON delta
    fn tool_use_delta(&self, partial_json: &str) -> String {
        format!(
            "event: content_block_delta\ndata: {}\n\n",
            json!({
                "type": "content_block_delta",
                "index": self.content_index,
                "delta": {
                    "type": "input_json_delta",
                    "partial_json": partial_json
                }
            })
        )
    }

    /// Generate message_delta (stop event)
    fn message_delta(&self, stop_reason: &str, output_tokens: u64) -> String {
        format!(
            "event: message_delta\ndata: {}\n\n",
            json!({
                "type": "message_delta",
                "delta": {
                    "stop_reason": stop_reason,
                    "stop_sequence": null
                },
                "usage": {
                    "output_tokens": output_tokens
                }
            })
        )
    }

    /// Generate message_stop
    fn message_stop(&self) -> String {
        "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_string()
    }

    /// Generate a ping event
    fn ping(&self) -> String {
        "event: ping\ndata: {\"type\":\"ping\"}\n\n".to_string()
    }

    /// Process a complete Gemini SSE data line and return Anthropic SSE events.
    ///
    /// A Gemini SSE event looks like:
    /// `data: {"candidates":[{"content":{"parts":[{"text":"Hello"}]},"finishReason":"STOP"}],...}`
    fn process_gemini_event(&mut self, data_json: &str) -> Vec<String> {
        let mut events = Vec::new();

        // Log raw Gemini SSE data for debugging
        let preview_len = data_json.len().min(500);
        tracing::debug!("[AnthropicBridge] Raw Gemini SSE ({} bytes): {}", data_json.len(), &data_json[..preview_len]);

        let parsed: Value = match serde_json::from_str(data_json) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("[AnthropicBridge] Failed to parse Gemini SSE: {} — data: {}...", e, &data_json[..data_json.len().min(200)]);
                return events;
            }
        };

        // Unwrap response wrapper if present
        let response_obj = parsed.get("response").unwrap_or(&parsed);

        // Log key fields for debugging
        let has_candidates = response_obj.get("candidates").is_some();
        let has_usage = response_obj.get("usageMetadata").is_some();
        let finish_reason = response_obj
            .get("candidates")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|c| c.get("finishReason"))
            .and_then(|f| f.as_str())
            .unwrap_or("none");
        tracing::info!(
            "[AnthropicBridge] Gemini event: candidates={}, usage={}, finish={}, started={}, content_index={}",
            has_candidates, has_usage, finish_reason, self.started, self.content_index
        );

        // Extract usage metadata
        if let Some(usage) = response_obj.get("usageMetadata") {
            let input = usage
                .get("promptTokenCount")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let output = usage
                .get("candidatesTokenCount")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            self.total_input_tokens = input;
            self.total_output_tokens = output;
        }

        // Emit message_start on first event
        if !self.started {
            let input_tokens = response_obj
                .get("usageMetadata")
                .and_then(|u| u.get("promptTokenCount"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            events.push(self.message_start(input_tokens));
            events.push(self.ping());
        }

        // Process candidates
        if let Some(candidates) = response_obj.get("candidates").and_then(|c| c.as_array()) {
            for candidate in candidates {
                if let Some(content) = candidate.get("content") {
                    if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                        for part in parts {
                            // Check if it's a thinking part
                            if part.get("thought").and_then(|t| t.as_bool()).unwrap_or(false) {
                                if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                    // If a different block type is open, close it first
                                    if self.current_block_type.as_deref() != Some("thinking") {
                                        if self.current_block_type.is_some() {
                                            events.push(self.content_block_stop());
                                        }
                                        events.push(self.thinking_block_start());
                                        self.current_block_type = Some("thinking".to_string());
                                    }
                                    events.push(self.thinking_delta(text));
                                }
                            } else if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                // Regular text part — reuse open text block or open a new one
                                if self.current_block_type.as_deref() != Some("text") {
                                    if self.current_block_type.is_some() {
                                        events.push(self.content_block_stop());
                                    }
                                    events.push(self.text_block_start());
                                    self.current_block_type = Some("text".to_string());
                                }
                                events.push(self.text_delta(text));
                            } else if let Some(fc) = part.get("functionCall") {
                                // Function call → tool_use (each tool is its own block)
                                if self.current_block_type.is_some() {
                                    events.push(self.content_block_stop());
                                }
                                let name = fc.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                                let args = fc.get("args").cloned().unwrap_or(json!({}));
                                let tool_id = format!("toolu_{}", &uuid::Uuid::new_v4().to_string().replace('-', "")[..24]);

                                events.push(self.tool_use_block_start(&tool_id, name));
                                let args_str = serde_json::to_string(&args).unwrap_or_default();
                                events.push(self.tool_use_delta(&args_str));
                                self.current_block_type = Some("tool_use".to_string());
                            }
                        }
                    }
                }

                // Check finish reason
                if let Some(finish) = candidate.get("finishReason").and_then(|f| f.as_str()) {
                    // Close any open content block before message_delta
                    if self.current_block_type.is_some() {
                        events.push(self.content_block_stop());
                        self.current_block_type = None;
                    }
                    let stop_reason = match finish {
                        "STOP" => {
                            if self.has_tool_use {
                                "tool_use"
                            } else {
                                "end_turn"
                            }
                        }
                        "MAX_TOKENS" => "max_tokens",
                        "SAFETY" => "end_turn",
                        _ => "end_turn",
                    };
                    events.push(self.message_delta(stop_reason, self.total_output_tokens));
                    events.push(self.message_stop());
                }
            }
        }

        events
    }

    /// Process raw bytes from the upstream Gemini SSE stream.
    /// Handles line buffering for incomplete chunks.
    fn process_chunk(&mut self, chunk: &[u8]) -> Vec<String> {
        let text = match std::str::from_utf8(chunk) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        tracing::debug!("[AnthropicBridge] Raw chunk ({} bytes): {}...", chunk.len(), &text[..text.len().min(200)]);
        self.line_buffer.push_str(text);

        let mut events = Vec::new();

        // Process complete SSE lines (separated by \n\n or \r\n\r\n)
        loop {
            // Find end of SSE event
            let boundary = if let Some(pos) = self.line_buffer.find("\n\n") {
                Some((pos, 2))
            } else if let Some(pos) = self.line_buffer.find("\r\n\r\n") {
                Some((pos, 4))
            } else {
                None
            };

            match boundary {
                Some((pos, sep_len)) => {
                    let event_text = self.line_buffer[..pos].to_string();
                    self.line_buffer = self.line_buffer[pos + sep_len..].to_string();

                    // Parse SSE event lines
                    for line in event_text.lines() {
                        let trimmed = line.trim();
                        if let Some(data) = trimmed.strip_prefix("data:") {
                            let data = data.trim();
                            if !data.is_empty() && data != "[DONE]" {
                                let mut new_events = self.process_gemini_event(data);
                                events.append(&mut new_events);
                            }
                        }
                    }
                }
                None => break,
            }
        }

        events
    }
}

/// Create a streaming response that transforms Gemini SSE → Anthropic SSE.
///
/// Takes a reqwest response with Gemini SSE body and returns an axum Body
/// that emits Anthropic SSE events.
pub fn create_anthropic_sse_stream(
    upstream_response: reqwest::Response,
    model: &str,
) -> axum::body::Body {
    let model = model.to_string();

    let stream = async_stream::stream! {
        let mut converter = AnthropicSseConverter::new(&model);
        let mut byte_stream = upstream_response.bytes_stream();
        let mut total_bytes: usize = 0;
        let mut chunk_count: usize = 0;

        while let Some(chunk_result) = byte_stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    chunk_count += 1;
                    total_bytes += chunk.len();
                    tracing::info!(
                        "[AnthropicBridge] Chunk #{}: {} bytes (total: {} bytes)",
                        chunk_count, chunk.len(), total_bytes
                    );
                    let events = converter.process_chunk(&chunk);
                    for event in events {
                        yield Ok::<Bytes, std::io::Error>(Bytes::from(event));
                    }
                }
                Err(e) => {
                    tracing::error!("[AnthropicBridge] Stream error: {}", e);
                    // Emit an error event and stop
                    let error_event = format!(
                        "event: error\ndata: {}\n\n",
                        json!({"type": "error", "error": {"type": "server_error", "message": format!("Upstream error: {}", e)}})
                    );
                    yield Ok::<Bytes, std::io::Error>(Bytes::from(error_event));
                    break;
                }
            }
        }

        tracing::info!(
            "[AnthropicBridge] Stream ended: {} chunks, {} total bytes, started={}, content_index={}, has_tool_use={}",
            chunk_count, total_bytes, converter.started, converter.content_index, converter.has_tool_use
        );

        // ── Post-stream finalizer ─────────────────────────────────
        // Ensure message_stop is ALWAYS sent when the SSE stream ends.
        // The Anthropic SDK's get_final_message() asserts that the
        // internal __final_message_snapshot is not None, which requires
        // at least message_start to have been processed. And the SDK
        // expects the sequence to end with message_delta + message_stop.
        //
        // Three cases:
        //   A) converter never started → emit full minimal response
        //   B) converter started, has open content block → close it + stop
        //   C) converter started, content properly closed but no stop → emit stop
        if !converter.started {
            // Case A: Gemini returned nothing parseable — emit minimal valid response
            tracing::warn!(
                "[AnthropicBridge] CASE A: Stream ended with NO parseable events (0 Gemini events). Emitting empty minimal response."
            );
            let events = vec![
                converter.message_start(0),
                converter.ping(),
                converter.text_block_start(),
                converter.text_delta(""),
                converter.content_block_stop(),
                converter.message_delta("end_turn", 0),
                converter.message_stop(),
            ];
            for event in events {
                yield Ok::<Bytes, std::io::Error>(Bytes::from(event));
            }
        } else if converter.current_block_type.is_some() {
            // Case B: Stream ended with an open content block (no finishReason from Gemini)
            tracing::info!("[AnthropicBridge] CASE B: Closing open content block");
            let stop_reason = if converter.has_tool_use { "tool_use" } else { "end_turn" };
            let events = vec![
                converter.content_block_stop(),
                converter.message_delta(stop_reason, converter.total_output_tokens),
                converter.message_stop(),
            ];
            for event in events {
                yield Ok::<Bytes, std::io::Error>(Bytes::from(event));
            }
        } else if converter.content_index == 0 {
            // Case C-1: Started but no content blocks at all — emit minimal content + stop
            tracing::info!("[AnthropicBridge] CASE C-1: No content blocks");
            let events = vec![
                converter.text_block_start(),
                converter.text_delta(""),
                converter.content_block_stop(),
                converter.message_delta("end_turn", converter.total_output_tokens),
                converter.message_stop(),
            ];
            for event in events {
                yield Ok::<Bytes, std::io::Error>(Bytes::from(event));
            }
        } else {
            tracing::info!("[AnthropicBridge] CASE C-2: Stream properly terminated");
        }
        // Case C-2: content_index > 0 && current_block_type is None && started
        // This means finishReason was already processed (which emits message_delta + message_stop)
        // so no action needed — the stream is already properly terminated.
    };

    axum::body::Body::from_stream(stream)
}

/// Buffer a Gemini SSE stream and produce a single non-streaming Anthropic response.
///
/// This is used when the client requests non-streaming, but we internally use
/// streamGenerateContent (because generateContent returns 500 for some models).
pub async fn buffer_sse_to_anthropic_response(
    upstream_response: reqwest::Response,
    model: &str,
) -> Value {
    let message_id = format!(
        "msg_{}",
        &uuid::Uuid::new_v4().to_string().replace('-', "")[..24]
    );

    let mut content_blocks: Vec<Value> = Vec::new();
    let mut stop_reason = "end_turn";
    let mut has_tool_use = false;
    let mut total_input_tokens: u64 = 0;
    let mut total_output_tokens: u64 = 0;

    // Buffers for accumulating text across multiple SSE events
    let mut current_text = String::new();
    let mut current_thinking = String::new();
    let mut line_buffer = String::new();

    let mut byte_stream = upstream_response.bytes_stream();
    let mut chunk_count: usize = 0;
    let mut total_bytes: usize = 0;

    while let Some(chunk_result) = byte_stream.next().await {
        let chunk = match chunk_result {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("[AnthropicBridge] Buffer stream error: {}", e);
                break;
            }
        };

        let text = match std::str::from_utf8(&chunk) {
            Ok(s) => s,
            Err(_) => continue,
        };

        chunk_count += 1;
        total_bytes += chunk.len();
        tracing::info!(
            "[AnthropicBridge/Buffer] Chunk #{}: {} bytes. Data: {}...",
            chunk_count, chunk.len(), &text[..text.len().min(300)]
        );

        line_buffer.push_str(text);

        // Process complete SSE events
        loop {
            let boundary = if let Some(pos) = line_buffer.find("\n\n") {
                Some((pos, 2))
            } else if let Some(pos) = line_buffer.find("\r\n\r\n") {
                Some((pos, 4))
            } else {
                None
            };

            match boundary {
                Some((pos, sep_len)) => {
                    let event_text = line_buffer[..pos].to_string();
                    line_buffer = line_buffer[pos + sep_len..].to_string();

                    for line in event_text.lines() {
                        let trimmed = line.trim();
                        if let Some(data) = trimmed.strip_prefix("data:") {
                            let data = data.trim();
                            if data.is_empty() || data == "[DONE]" {
                                continue;
                            }

                            let parsed: Value = match serde_json::from_str(data) {
                                Ok(v) => v,
                                Err(_) => continue,
                            };

                            let response_obj = parsed.get("response").unwrap_or(&parsed);

                            // Update usage
                            if let Some(usage) = response_obj.get("usageMetadata") {
                                if let Some(v) = usage.get("promptTokenCount").and_then(|v| v.as_u64()) {
                                    total_input_tokens = v;
                                }
                                if let Some(v) = usage.get("candidatesTokenCount").and_then(|v| v.as_u64()) {
                                    total_output_tokens = v;
                                }
                            }

                            // Process candidates
                            if let Some(candidates) = response_obj.get("candidates").and_then(|c| c.as_array()) {
                                for candidate in candidates {
                                    if let Some(content) = candidate.get("content") {
                                        if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                                            for part in parts {
                                                if part.get("thought").and_then(|t| t.as_bool()).unwrap_or(false) {
                                                    // Thinking part — flush any accumulated text first
                                                    if !current_text.is_empty() {
                                                        content_blocks.push(json!({"type": "text", "text": current_text}));
                                                        current_text.clear();
                                                    }
                                                    if let Some(t) = part.get("text").and_then(|t| t.as_str()) {
                                                        current_thinking.push_str(t);
                                                    }
                                                } else if let Some(t) = part.get("text").and_then(|t| t.as_str()) {
                                                    // Regular text — flush any accumulated thinking first
                                                    if !current_thinking.is_empty() {
                                                        content_blocks.push(json!({"type": "thinking", "thinking": current_thinking}));
                                                        current_thinking.clear();
                                                    }
                                                    current_text.push_str(t);
                                                } else if let Some(fc) = part.get("functionCall") {
                                                    // Flush accumulated text/thinking
                                                    if !current_thinking.is_empty() {
                                                        content_blocks.push(json!({"type": "thinking", "thinking": current_thinking}));
                                                        current_thinking.clear();
                                                    }
                                                    if !current_text.is_empty() {
                                                        content_blocks.push(json!({"type": "text", "text": current_text}));
                                                        current_text.clear();
                                                    }
                                                    has_tool_use = true;
                                                    let name = fc.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                                                    let args = fc.get("args").cloned().unwrap_or(json!({}));
                                                    let tool_id = format!("toolu_{}", &uuid::Uuid::new_v4().to_string().replace('-', "")[..24]);
                                                    content_blocks.push(json!({
                                                        "type": "tool_use",
                                                        "id": tool_id,
                                                        "name": name,
                                                        "input": args
                                                    }));
                                                }
                                            }
                                        }
                                    }

                                    if let Some(finish) = candidate.get("finishReason").and_then(|f| f.as_str()) {
                                        stop_reason = match finish {
                                            "STOP" => {
                                                if has_tool_use { "tool_use" } else { "end_turn" }
                                            }
                                            "MAX_TOKENS" => "max_tokens",
                                            _ => "end_turn",
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
                None => break,
            }
        }
    }

    // Flush remaining accumulated content
    if !current_thinking.is_empty() {
        content_blocks.push(json!({"type": "thinking", "thinking": current_thinking}));
    }
    if !current_text.is_empty() {
        content_blocks.push(json!({"type": "text", "text": current_text}));
    }

    json!({
        "id": message_id,
        "type": "message",
        "role": "assistant",
        "content": content_blocks,
        "model": model,
        "stop_reason": stop_reason,
        "stop_sequence": null,
        "usage": {
            "input_tokens": total_input_tokens,
            "output_tokens": total_output_tokens
        }
    })
}

/// Create a non-streaming Anthropic response from a Gemini response body.
pub fn create_anthropic_response(gemini_body: &str, model: &str) -> Value {
    let parsed: Value = serde_json::from_str(gemini_body).unwrap_or(json!({}));

    let message_id = format!(
        "msg_{}",
        &uuid::Uuid::new_v4().to_string().replace('-', "")[..24]
    );

    // Extract text and tool calls from Gemini response
    let mut content_blocks: Vec<Value> = Vec::new();
    let mut stop_reason = "end_turn";
    let mut has_tool_use = false;
    // Gemini v1internal wraps response: {"response": {"candidates": [...]}} or direct {"candidates": [...]}
    let response_obj = parsed.get("response").unwrap_or(&parsed);

    if let Some(candidates) = response_obj.get("candidates").and_then(|c| c.as_array()) {
        for candidate in candidates {
            if let Some(content) = candidate.get("content") {
                if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                    for part in parts {
                        if part.get("thought").and_then(|t| t.as_bool()).unwrap_or(false) {
                            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                content_blocks.push(json!({
                                    "type": "thinking",
                                    "thinking": text
                                }));
                            }
                        } else if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                            content_blocks.push(json!({
                                "type": "text",
                                "text": text
                            }));
                        } else if let Some(fc) = part.get("functionCall") {
                            has_tool_use = true;
                            let name = fc.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                            let args = fc.get("args").cloned().unwrap_or(json!({}));
                            let tool_id = format!(
                                "toolu_{}",
                                &uuid::Uuid::new_v4().to_string().replace('-', "")[..24]
                            );
                            content_blocks.push(json!({
                                "type": "tool_use",
                                "id": tool_id,
                                "name": name,
                                "input": args
                            }));
                        }
                    }
                }
            }

            if let Some(finish) = candidate.get("finishReason").and_then(|f| f.as_str()) {
                stop_reason = match finish {
                    "STOP" => {
                        if has_tool_use {
                            "tool_use"
                        } else {
                            "end_turn"
                        }
                    }
                    "MAX_TOKENS" => "max_tokens",
                    _ => "end_turn",
                };
            }
        }
    }

    // Extract usage
    let input_tokens = response_obj
        .get("usageMetadata")
        .and_then(|u| u.get("promptTokenCount"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let output_tokens = response_obj
        .get("usageMetadata")
        .and_then(|u| u.get("candidatesTokenCount"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    json!({
        "id": message_id,
        "type": "message",
        "role": "assistant",
        "content": content_blocks,
        "model": model,
        "stop_reason": stop_reason,
        "stop_sequence": null,
        "usage": {
            "input_tokens": input_tokens,
            "output_tokens": output_tokens
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_anthropic_format_by_system() {
        let body = json!({"model": "claude-opus-4-6-thinking", "system": "You are helpful", "messages": []});
        let headers = axum::http::HeaderMap::new();
        assert!(is_anthropic_format(&body, &headers));
    }

    #[test]
    fn test_detect_openai_format() {
        let body = json!({"model": "gpt-4", "messages": [{"role": "system", "content": "You are helpful"}]});
        let headers = axum::http::HeaderMap::new();
        assert!(!is_anthropic_format(&body, &headers));
    }

    #[test]
    fn test_detect_anthropic_format_by_tools() {
        let body = json!({
            "model": "claude-sonnet-4-20250514",
            "tools": [{"name": "test", "description": "test", "input_schema": {"type": "object"}}],
            "messages": []
        });
        let headers = axum::http::HeaderMap::new();
        assert!(is_anthropic_format(&body, &headers));
    }

    #[test]
    fn test_extract_system_string() {
        let body = json!({"system": "You are a helpful assistant"});
        let result = extract_system_instruction(&body);
        assert!(result.is_some());
        let parts = result.unwrap();
        assert_eq!(
            parts["parts"][0]["text"].as_str().unwrap(),
            "You are a helpful assistant"
        );
    }

    #[test]
    fn test_convert_tools() {
        let body = json!({
            "tools": [{
                "name": "grep_search",
                "description": "Search files",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"}
                    },
                    "required": ["query"]
                }
            }]
        });
        let result = convert_anthropic_tools(&body);
        assert!(result.is_some());
        let tools = result.unwrap();
        let decls = tools[0]["functionDeclarations"].as_array().unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0]["name"].as_str().unwrap(), "grep_search");
    }
}
