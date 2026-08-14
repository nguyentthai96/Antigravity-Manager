use serde::{Deserialize, Serialize};

// Google OAuth configuration
const CLIENT_ID: &str = "1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com";
const CLIENT_SECRET: &str = "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";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";
const TOKEN_REFRESH_SKEW_SECONDS: i64 = 900;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    #[serde(default)]
    pub token_type: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub email: String,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
}

/// Get HTTP client for OAuth requests
fn get_http_client() -> reqwest::Client {
    crate::utils::http::get_long_client()
}

/// Refresh access_token using refresh_token
pub async fn refresh_access_token(
    refresh_token: &str,
    account_id: Option<&str>,
) -> Result<TokenResponse, String> {
    let client = get_http_client();

    let params = [
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    if let Some(id) = account_id {
        crate::logger::log_info(&format!("Refreshing Token for account: {}...", id));
    }

    let response = client
        .post(TOKEN_URL)
        .header(
            reqwest::header::USER_AGENT,
            crate::constants::NATIVE_OAUTH_USER_AGENT.as_str(),
        )
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() || e.is_timeout() {
                format!(
                    "Refresh request failed: {}. Cannot connect to Google auth server.",
                    e
                )
            } else {
                format!("Refresh request failed: {}", e)
            }
        })?;

    if response.status().is_success() {
        let token_data = response
            .json::<TokenResponse>()
            .await
            .map_err(|e| format!("Refresh data parsing failed: {}", e))?;

        crate::logger::log_info(&format!(
            "Token refreshed successfully! Expires in: {} seconds",
            token_data.expires_in
        ));

        Ok(token_data)
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Refresh failed ({}): {}", status, error_text))
    }
}

/// Get user info
pub async fn get_user_info(access_token: &str) -> Result<UserInfo, String> {
    let client = crate::utils::http::get_client();

    let response = client
        .get(USERINFO_URL)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("User info request failed: {}", e))?;

    if response.status().is_success() {
        response
            .json::<UserInfo>()
            .await
            .map_err(|e| format!("User info parsing failed: {}", e))
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Failed to get user info: {}", error_text))
    }
}

/// Check and refresh Token if needed
pub async fn ensure_fresh_token(
    current_token: &crate::models::TokenData,
    account_id: Option<&str>,
) -> Result<crate::models::TokenData, String> {
    let now = chrono::Local::now().timestamp();

    if current_token.expiry_timestamp > now + TOKEN_REFRESH_SKEW_SECONDS {
        return Ok(current_token.clone());
    }

    crate::logger::log_info(&format!(
        "Token expiring soon for account {:?}, refreshing...",
        account_id
    ));

    let response = refresh_access_token(&current_token.refresh_token, account_id).await?;

    Ok(crate::models::TokenData::new(
        response.access_token,
        current_token.refresh_token.clone(),
        response.expires_in,
        current_token.email.clone(),
        current_token.project_id.clone(),
        None,
        current_token.is_gcp_tos,
        response.id_token.or(current_token.id_token.clone()),
    ))
}
