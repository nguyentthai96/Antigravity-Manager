use super::{quota::QuotaData, token::TokenData};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveLimitStatus {
    pub model: String,
    pub status: u16,
    pub reason: String,
    pub until: i64,
    pub detected_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Account data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub token: TokenData,
    pub quota: Option<QuotaData>,
    /// Disabled accounts are ignored by the proxy token pool
    #[serde(default)]
    pub disabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled_at: Option<i64>,
    /// User manually disabled proxy feature
    #[serde(default)]
    pub proxy_disabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_disabled_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_disabled_at: Option<i64>,
    /// Quota-protected disabled models
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub protected_models: HashSet<String>,
    /// Temporary live upstream throttles
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub live_limited_models: HashMap<String, LiveLimitStatus>,
    /// 403 validation blocked status
    #[serde(default)]
    pub validation_blocked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_blocked_until: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_blocked_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_url: Option<String>,
    pub created_at: i64,
    pub last_used: i64,
    /// Bound proxy ID (None = use global proxy pool)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_bound_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_label: Option<String>,
    /// Skip x-goog-user-project header (cached after 403 SERVICE_DISABLED)
    #[serde(default)]
    pub skip_project_header: bool,
}

impl Account {
    pub fn new(id: String, email: String, token: TokenData) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            email,
            name: None,
            token,
            quota: None,
            disabled: false,
            disabled_reason: None,
            disabled_at: None,
            proxy_disabled: false,
            proxy_disabled_reason: None,
            proxy_disabled_at: None,
            protected_models: HashSet::new(),
            live_limited_models: HashMap::new(),
            validation_blocked: false,
            validation_blocked_until: None,
            validation_blocked_reason: None,
            validation_url: None,
            created_at: now,
            last_used: now,
            proxy_id: None,
            proxy_bound_at: None,
            custom_label: None,
            skip_project_header: false,
        }
    }

    pub fn update_last_used(&mut self) {
        self.last_used = chrono::Utc::now().timestamp();
    }

    pub fn update_quota(&mut self, mut quota: QuotaData) {
        if let Some(ref existing) = self.quota {
            if quota.subscription_tier.is_none() {
                quota.subscription_tier = existing.subscription_tier.clone();
            }
        }
        self.quota = Some(quota);
    }
}

/// Export account item (for backup/migration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountExportItem {
    pub email: String,
    pub refresh_token: String,
}
