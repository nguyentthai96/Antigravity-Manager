use once_cell::sync::Lazy;
use std::time::Duration;

/// Standard HTTP client (15s timeout)
static STANDARD_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(10))
        .pool_max_idle_per_host(10)
        .build()
        .expect("Failed to create HTTP client")
});

/// Long-timeout HTTP client (60s timeout)
static LONG_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(15))
        .pool_max_idle_per_host(10)
        .build()
        .expect("Failed to create long HTTP client")
});

/// Streaming client (no timeout — for SSE streams)
static STREAMING_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .pool_max_idle_per_host(10)
        .build()
        .expect("Failed to create streaming HTTP client")
});

pub fn get_client() -> reqwest::Client {
    STANDARD_CLIENT.clone()
}

pub fn get_long_client() -> reqwest::Client {
    LONG_CLIENT.clone()
}

pub fn get_streaming_client() -> reqwest::Client {
    STREAMING_CLIENT.clone()
}
