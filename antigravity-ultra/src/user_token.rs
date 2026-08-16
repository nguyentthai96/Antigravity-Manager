//! User Token Database Module — standalone version

use chrono::{FixedOffset, Timelike, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// User token structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserToken {
    pub id: String,
    pub token: String,
    pub username: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub expires_type: String,
    pub expires_at: Option<i64>,
    pub max_ips: i32,
    pub curfew_start: Option<String>,
    pub curfew_end: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_used_at: Option<i64>,
    pub total_requests: i64,
    pub total_tokens_used: i64,
}

fn get_db_path() -> Result<PathBuf, String> {
    let mut path = crate::config::get_data_dir();
    path.push("user_tokens.db");
    Ok(path)
}

fn connect_db() -> Result<Connection, String> {
    let path = get_db_path()?;
    Connection::open(&path).map_err(|e| format!("Failed to open database: {}", e))
}

/// Initialize database
pub fn init_db() -> Result<(), String> {
    let conn = connect_db()?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_tokens (
            id TEXT PRIMARY KEY,
            token TEXT UNIQUE NOT NULL,
            username TEXT NOT NULL,
            description TEXT,
            enabled BOOLEAN NOT NULL DEFAULT 1,
            expires_type TEXT NOT NULL,
            expires_at INTEGER,
            max_ips INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            last_used_at INTEGER,
            total_requests INTEGER NOT NULL DEFAULT 0,
            total_tokens_used INTEGER NOT NULL DEFAULT 0,
            curfew_start TEXT,
            curfew_end TEXT
        )",
        [],
    )
    .map_err(|e| format!("Failed to create user_tokens table: {}", e))?;

    // Migration for old DBs
    let _ = conn.execute("ALTER TABLE user_tokens ADD COLUMN expires_type TEXT", []);
    let _ = conn.execute("ALTER TABLE user_tokens ADD COLUMN expires_at INTEGER", []);
    let _ = conn.execute("ALTER TABLE user_tokens ADD COLUMN max_ips INTEGER DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE user_tokens ADD COLUMN total_requests INTEGER DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE user_tokens ADD COLUMN total_tokens_used INTEGER DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE user_tokens ADD COLUMN last_used_at INTEGER", []);
    let _ = conn.execute("ALTER TABLE user_tokens ADD COLUMN curfew_start TEXT", []);
    let _ = conn.execute("ALTER TABLE user_tokens ADD COLUMN curfew_end TEXT", []);

    // IP bindings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS token_ip_bindings (
            id TEXT PRIMARY KEY,
            token_id TEXT NOT NULL,
            ip_address TEXT NOT NULL,
            first_seen_at INTEGER NOT NULL,
            last_seen_at INTEGER NOT NULL,
            request_count INTEGER NOT NULL DEFAULT 0,
            user_agent TEXT,
            FOREIGN KEY(token_id) REFERENCES user_tokens(id) ON DELETE CASCADE,
            UNIQUE(token_id, ip_address)
        )",
        [],
    )
    .map_err(|e| format!("Failed to create token_ip_bindings table: {}", e))?;

    // Data sanitization
    let _ = conn.execute("UPDATE user_tokens SET expires_type = 'never' WHERE expires_type IS NULL OR expires_type = ''", []);
    let _ = conn.execute("UPDATE user_tokens SET max_ips = 0 WHERE max_ips IS NULL", []);
    let _ = conn.execute("UPDATE user_tokens SET total_requests = 0 WHERE total_requests IS NULL", []);
    let _ = conn.execute("UPDATE user_tokens SET total_tokens_used = 0 WHERE total_tokens_used IS NULL", []);
    let _ = conn.execute("UPDATE user_tokens SET enabled = 1 WHERE enabled IS NULL", []);

    Ok(())
}

/// Create new token
pub fn create_token(
    username: String,
    expires_type: String,
    description: Option<String>,
    max_ips: i32,
    curfew_start: Option<String>,
    curfew_end: Option<String>,
    custom_expires_at: Option<i64>,
    custom_token: Option<String>,
) -> Result<UserToken, String> {
    let conn = connect_db()?;
    let id = Uuid::new_v4().to_string();
    let token = match custom_token {
        Some(t) if !t.is_empty() => {
            if t.starts_with("sk-") { t } else { format!("sk-{}", t) }
        }
        _ => format!("sk-{}", Uuid::new_v4().to_string().replace('-', "")),
    };
    let now = Utc::now().timestamp();

    let expires_at = match expires_type.as_str() {
        "day" => Some(Utc::now().checked_add_signed(chrono::Duration::days(1)).unwrap().timestamp()),
        "week" => Some(Utc::now().checked_add_signed(chrono::Duration::weeks(1)).unwrap().timestamp()),
        "month" => Some(Utc::now().checked_add_signed(chrono::Duration::days(30)).unwrap().timestamp()),
        "custom" => custom_expires_at,
        _ => None,
    };

    let user_token = UserToken {
        id: id.clone(),
        token: token.clone(),
        username: username.clone(),
        description: description.clone(),
        enabled: true,
        expires_type: expires_type.clone(),
        expires_at,
        max_ips,
        curfew_start: curfew_start.clone(),
        curfew_end: curfew_end.clone(),
        created_at: now,
        updated_at: now,
        last_used_at: None,
        total_requests: 0,
        total_tokens_used: 0,
    };

    conn.execute(
        "INSERT INTO user_tokens (
            id, token, username, description, enabled, expires_type, expires_at, max_ips,
            curfew_start, curfew_end,
            created_at, updated_at, total_requests, total_tokens_used
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            user_token.id,
            user_token.token,
            user_token.username,
            user_token.description,
            user_token.enabled,
            user_token.expires_type,
            user_token.expires_at,
            user_token.max_ips,
            user_token.curfew_start,
            user_token.curfew_end,
            user_token.created_at,
            user_token.updated_at,
            user_token.total_requests,
            user_token.total_tokens_used,
        ],
    )
    .map_err(|e| format!("Failed to insert user token: {}", e))?;

    Ok(user_token)
}

/// List all tokens
pub fn list_tokens() -> Result<Vec<UserToken>, String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare("SELECT * FROM user_tokens ORDER BY created_at DESC")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let token_iter = stmt
        .query_map([], |row| {
            Ok(UserToken {
                id: row.get("id")?,
                token: row.get("token")?,
                username: row.get("username")?,
                description: row.get("description")?,
                enabled: row.get("enabled").unwrap_or(true),
                expires_type: row.get("expires_type").unwrap_or("never".to_string()),
                expires_at: row.get("expires_at").unwrap_or(None),
                max_ips: row.get("max_ips").unwrap_or(0),
                curfew_start: row.get("curfew_start").unwrap_or(None),
                curfew_end: row.get("curfew_end").unwrap_or(None),
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
                last_used_at: row.get("last_used_at").unwrap_or(None),
                total_requests: row.get("total_requests").unwrap_or(0),
                total_tokens_used: row.get("total_tokens_used").unwrap_or(0),
            })
        })
        .map_err(|e| format!("Failed to query tokens: {}", e))?;

    let mut tokens = Vec::new();
    for token in token_iter {
        tokens.push(token.map_err(|e| format!("Failed to parse token row: {}", e))?);
    }

    Ok(tokens)
}

/// Get token by value (for API key validation)
pub fn get_token_by_value(token: &str) -> Result<Option<UserToken>, String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare("SELECT * FROM user_tokens WHERE token = ?1")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let result = stmt
        .query_row(params![token], |row| {
            Ok(UserToken {
                id: row.get("id")?,
                token: row.get("token")?,
                username: row.get("username")?,
                description: row.get("description")?,
                enabled: row.get("enabled")?,
                expires_type: row.get("expires_type")?,
                expires_at: row.get("expires_at")?,
                max_ips: row.get("max_ips")?,
                curfew_start: row.get("curfew_start").unwrap_or(None),
                curfew_end: row.get("curfew_end").unwrap_or(None),
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
                last_used_at: row.get("last_used_at")?,
                total_requests: row.get("total_requests")?,
                total_tokens_used: row.get("total_tokens_used")?,
            })
        })
        .optional()
        .map_err(|e| format!("Failed to query token: {}", e))?;

    Ok(result)
}

/// Delete token
pub fn delete_token(id: &str) -> Result<(), String> {
    let conn = connect_db()?;
    conn.execute("DELETE FROM user_tokens WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete token: {}", e))?;
    Ok(())
}

/// Validate token (expiry + IP limit + curfew)
pub fn validate_token(token_str: &str, ip: &str) -> Result<(bool, Option<String>), String> {
    let token_opt = get_token_by_value(token_str)?;

    if let Some(token) = token_opt {
        if !token.enabled {
            return Ok((false, Some("Token is disabled.".to_string())));
        }

        // 1. Check expiry
        if token.expires_type != "never" {
            if let Some(expires_at) = token.expires_at {
                if expires_at < Utc::now().timestamp() {
                    return Ok((false, Some("Your token has expired.".to_string())));
                }
            }
        }

        // 2. IP limit
        if token.max_ips > 0 {
            let conn = connect_db()?;
            let is_bound: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM token_ip_bindings WHERE token_id = ?1 AND ip_address = ?2)",
                params![token.id, ip],
                |row| row.get(0),
            ).unwrap_or(false);

            if !is_bound {
                let current_ip_count: i32 = conn.query_row(
                    "SELECT COUNT(*) FROM token_ip_bindings WHERE token_id = ?1",
                    params![token.id],
                    |row| row.get(0),
                ).unwrap_or(0);

                if current_ip_count >= token.max_ips {
                    return Ok((false, Some(format!("IP limit reached ({}/{})", current_ip_count, token.max_ips))));
                }
            }
        }

        // 3. Curfew check (Beijing time UTC+8)
        if let (Some(start_str), Some(end_str)) = (&token.curfew_start, &token.curfew_end) {
            if !start_str.is_empty() && !end_str.is_empty() {
                let beijing_offset = FixedOffset::east_opt(8 * 3600).unwrap();
                let now_beijing = Utc::now().with_timezone(&beijing_offset);
                let current_time_str = format!("{:02}:{:02}", now_beijing.hour(), now_beijing.minute());

                let is_curfew = if start_str > end_str {
                    current_time_str >= *start_str || current_time_str < *end_str
                } else {
                    current_time_str >= *start_str && current_time_str < *end_str
                };

                if is_curfew {
                    return Ok((false, Some(format!(
                        "Service not available between {} and {} Beijing Time.",
                        start_str, end_str
                    ))));
                }
            }
        }

        Ok((true, None))
    } else {
        Ok((false, Some("Invalid token.".to_string())))
    }
}

/// Record token usage and update IP binding
pub fn record_token_usage(
    token_id: &str,
    ip: &str,
    _model: &str,
    _input_tokens: i32,
    _output_tokens: i32,
    user_agent: Option<String>,
) -> Result<(), String> {
    let mut conn = connect_db()?;
    let tx = conn.transaction().map_err(|e| format!("Transaction failed: {}", e))?;
    let now = Utc::now().timestamp();

    tx.execute(
        "UPDATE user_tokens SET last_used_at = ?1, total_requests = total_requests + 1 WHERE id = ?2",
        params![now, token_id],
    ).map_err(|e| format!("Failed to update stats: {}", e))?;

    let binding_exists: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM token_ip_bindings WHERE token_id = ?1 AND ip_address = ?2)",
        params![token_id, ip],
        |row| row.get(0),
    ).unwrap_or(false);

    if binding_exists {
        tx.execute(
            "UPDATE token_ip_bindings SET last_seen_at = ?1, request_count = request_count + 1, user_agent = COALESCE(?2, user_agent) WHERE token_id = ?3 AND ip_address = ?4",
            params![now, user_agent, token_id, ip],
        ).map_err(|e| format!("Failed to update IP binding: {}", e))?;
    } else {
        let binding_id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO token_ip_bindings (id, token_id, ip_address, first_seen_at, last_seen_at, request_count, user_agent) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
            params![binding_id, token_id, ip, now, now, user_agent],
        ).map_err(|e| format!("Failed to insert IP binding: {}", e))?;
    }

    tx.commit().map_err(|e| format!("Commit failed: {}", e))?;
    Ok(())
}
