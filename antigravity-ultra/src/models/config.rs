use serde::{Deserialize, Serialize};

/// Proxy server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub port: u16,
    #[serde(default)]
    pub lan: bool,
    #[serde(default = "default_auth_mode")]
    pub auth_mode: String,
}

fn default_auth_mode() -> String {
    "token".to_string()
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            port: 8045,
            lan: false,
            auth_mode: default_auth_mode(),
        }
    }
}

/// Quota health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaHealthCheckConfig {
    pub enabled: bool,
    pub interval_seconds: u64,
}

impl Default for QuotaHealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 600,
        }
    }
}
