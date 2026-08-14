use crate::models::{ModelQuota, QuotaData, QuotaBucket, QuotaGroup};
use serde::{Deserialize, Serialize};
use serde_json::json;

// Quota API endpoints (fallback order)
const QUOTA_API_ENDPOINTS: [&str; 3] = [
    "https://daily-cloudcode-pa.sandbox.googleapis.com/v1internal:fetchAvailableModels",
    "https://daily-cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels",
    "https://cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels",
];

const QUOTA_SUMMARY_ENDPOINTS: [&str; 3] = [
    "https://daily-cloudcode-pa.sandbox.googleapis.com/v1internal:retrieveUserQuotaSummary",
    "https://daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary",
    "https://cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary",
];

const CLOUD_CODE_BASE_URL: &str = "https://daily-cloudcode-pa.sandbox.googleapis.com";

#[derive(Debug, Serialize, Deserialize)]
struct QuotaResponse {
    models: std::collections::HashMap<String, ModelInfo>,
    #[serde(rename = "deprecatedModelIds")]
    deprecated_model_ids: Option<std::collections::HashMap<String, DeprecatedModelInfo>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeprecatedModelInfo {
    #[serde(rename = "newModelId")]
    new_model_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelInfo {
    #[serde(rename = "quotaInfo")]
    quota_info: Option<QuotaInfo>,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "supportsImages")]
    supports_images: Option<bool>,
    #[serde(rename = "supportsThinking")]
    supports_thinking: Option<bool>,
    #[serde(rename = "thinkingBudget")]
    thinking_budget: Option<i32>,
    recommended: Option<bool>,
    #[serde(rename = "maxTokens")]
    max_tokens: Option<i32>,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: Option<i32>,
    #[serde(rename = "supportedMimeTypes")]
    supported_mime_types: Option<std::collections::HashMap<String, bool>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QuotaInfo {
    #[serde(rename = "remainingFraction")]
    remaining_fraction: Option<f64>,
    #[serde(rename = "resetTime")]
    reset_time: Option<String>,
}

#[derive(Debug, Deserialize)]
struct QuotaSummaryResponse {
    groups: Vec<QuotaSummaryGroup>,
}

#[derive(Debug, Deserialize)]
struct QuotaSummaryGroup {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    description: Option<String>,
    buckets: Vec<QuotaSummaryBucket>,
}

#[derive(Debug, Deserialize)]
struct QuotaSummaryBucket {
    #[serde(rename = "bucketId")]
    bucket_id: Option<String>,
    window: Option<String>,
    #[serde(rename = "remainingFraction")]
    remaining_fraction: Option<f64>,
    #[serde(rename = "resetTime")]
    reset_time: Option<String>,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LoadProjectResponse {
    #[serde(rename = "cloudaicompanionProject")]
    project_id: Option<String>,
    #[serde(rename = "currentTier")]
    current_tier: Option<Tier>,
    #[serde(rename = "paidTier")]
    paid_tier: Option<Tier>,
}

#[derive(Debug, Deserialize)]
struct Tier {
    id: Option<String>,
    name: Option<String>,
}

fn get_client() -> reqwest::Client {
    crate::utils::http::get_client()
}

/// Fetch project ID and subscription tier
async fn fetch_project_id(
    access_token: &str,
    email: &str,
) -> (Option<String>, Option<String>) {
    let client = get_client();
    let meta = json!({"metadata": {"ideType": "ANTIGRAVITY"}});

    let res = client
        .post(format!("{}/v1internal:loadCodeAssist", CLOUD_CODE_BASE_URL))
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", access_token))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::USER_AGENT, crate::constants::NATIVE_OAUTH_USER_AGENT.as_str())
        .json(&meta)
        .send()
        .await;

    match res {
        Ok(res) => {
            if res.status().is_success() {
                if let Ok(data) = res.json::<LoadProjectResponse>().await {
                    let project_id = data.project_id.clone();
                    let subscription_tier = data.paid_tier
                        .as_ref()
                        .and_then(|t| t.name.clone())
                        .or_else(|| data.paid_tier.as_ref().and_then(|t| t.id.clone()))
                        .or_else(|| data.current_tier.as_ref().and_then(|t| t.name.clone()))
                        .or_else(|| data.current_tier.as_ref().and_then(|t| t.id.clone()));

                    if let Some(ref tier) = subscription_tier {
                        crate::logger::log_info(&format!("📊 [{}] Subscription: {}", email, tier));
                    }

                    return (project_id, subscription_tier);
                }
            }
        }
        Err(e) => {
            crate::logger::log_warn(&format!("❌ [{}] loadCodeAssist error: {}", email, e));
        }
    }

    (None, None)
}

/// Fetch grouped quota summary
async fn fetch_quota_summary(
    access_token: &str,
    _email: &str,
    project_id: Option<&str>,
) -> Option<Vec<QuotaGroup>> {
    let client = get_client();
    let payload = if let Some(pid) = project_id {
        json!({ "project": pid })
    } else {
        json!({})
    };

    for ep_url in QUOTA_SUMMARY_ENDPOINTS.iter() {
        let res = client
            .post(*ep_url)
            .bearer_auth(access_token)
            .header(reqwest::header::USER_AGENT, crate::constants::NATIVE_OAUTH_USER_AGENT.as_str())
            .json(&payload)
            .send()
            .await;

        match res {
            Ok(response) => {
                if !response.status().is_success() {
                    if response.status().is_client_error() && response.status() != reqwest::StatusCode::TOO_MANY_REQUESTS {
                        return None;
                    }
                    continue;
                }

                let summary: QuotaSummaryResponse = match response.json().await {
                    Ok(s) => s,
                    Err(_) => return None,
                };

                let groups: Vec<QuotaGroup> = summary.groups.into_iter().map(|g| QuotaGroup {
                    display_name: g.display_name.unwrap_or_default(),
                    description: g.description,
                    buckets: g.buckets.into_iter().map(|b| QuotaBucket {
                        bucket_id: b.bucket_id.unwrap_or_default(),
                        window: b.window.unwrap_or_default(),
                        remaining_fraction: b.remaining_fraction.unwrap_or(0.0),
                        reset_time: b.reset_time.unwrap_or_default(),
                        display_name: b.display_name,
                        description: b.description,
                    }).collect(),
                }).collect();

                return Some(groups);
            }
            Err(_) => continue,
        }
    }

    None
}

/// Unified entry point for fetching account quota
pub async fn fetch_quota(
    access_token: &str,
    email: &str,
    cached_project_id: Option<&str>,
) -> crate::error::AppResult<(QuotaData, Option<String>)> {
    use crate::error::AppError;

    let (project_id, subscription_tier) = if let Some(pid) = cached_project_id {
        (Some(pid.to_string()), None)
    } else {
        fetch_project_id(access_token, email).await
    };

    let client = get_client();
    let payload = if let Some(ref pid) = project_id {
        json!({ "project": pid })
    } else {
        json!({})
    };

    let mut last_error: Option<AppError> = None;

    for (ep_idx, ep_url) in QUOTA_API_ENDPOINTS.iter().enumerate() {
        let has_next = ep_idx + 1 < QUOTA_API_ENDPOINTS.len();

        match client
            .post(*ep_url)
            .bearer_auth(access_token)
            .header(reqwest::header::USER_AGENT, crate::constants::NATIVE_OAUTH_USER_AGENT.as_str())
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => {
                if let Err(_) = response.error_for_status_ref() {
                    let status = response.status();

                    if status == reqwest::StatusCode::FORBIDDEN {
                        let mut q = QuotaData::new();
                        q.is_forbidden = true;
                        q.subscription_tier = subscription_tier.clone();
                        return Ok((q, project_id.clone()));
                    }

                    let text = response.text().await.unwrap_or_default();

                    if has_next && (status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error()) {
                        last_error = Some(AppError::Unknown(format!("HTTP {} - {}", status, text)));
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        continue;
                    }

                    return Err(AppError::Unknown(format!("API Error: {} - {}", status, text)));
                }

                let quota_response: QuotaResponse = response.json().await.map_err(AppError::from)?;
                let mut quota_data = QuotaData::new();

                for (name, info) in quota_response.models {
                    if let Some(quota_info) = info.quota_info {
                        let percentage = quota_info.remaining_fraction
                            .map(|f| (f * 100.0) as i32)
                            .unwrap_or(0);
                        let reset_time = quota_info.reset_time.clone().unwrap_or_default();

                        if name.starts_with("gemini") || name.starts_with("claude")
                            || name.starts_with("gpt") || name.starts_with("image")
                            || name.starts_with("imagen")
                        {
                            let model_quota = ModelQuota {
                                name,
                                percentage,
                                reset_time,
                                display_name: info.display_name,
                                supports_images: info.supports_images,
                                supports_thinking: info.supports_thinking,
                                thinking_budget: info.thinking_budget,
                                recommended: info.recommended,
                                max_tokens: info.max_tokens,
                                max_output_tokens: info.max_output_tokens,
                                supported_mime_types: info.supported_mime_types,
                            };
                            quota_data.add_model(model_quota);
                        }
                    }
                }

                if let Some(deprecated) = quota_response.deprecated_model_ids {
                    for (old_id, info) in deprecated {
                        quota_data.model_forwarding_rules.insert(old_id, info.new_model_id);
                    }
                }

                quota_data.subscription_tier = subscription_tier.clone();
                quota_data.quota_groups = fetch_quota_summary(access_token, email, project_id.as_deref()).await;

                return Ok((quota_data, project_id.clone()));
            }
            Err(e) => {
                last_error = Some(AppError::from(e));
                if has_next {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                continue;
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        AppError::Unknown("Quota fetch failed: all endpoints exhausted".to_string())
    }))
}
