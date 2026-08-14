use regex::Regex;
use std::sync::LazyLock;

const KNOWN_STABLE_VERSION: &str = "4.3.0";
const KNOWN_STABLE_ELECTRON: &str = "39.2.3";
const KNOWN_STABLE_CHROME: &str = "132.0.6834.160";

static VERSION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\d+\.\d+\.\d+").expect("Invalid version regex"));

fn parse_version(text: &str) -> Option<String> {
    VERSION_REGEX.find(text).map(|m| m.as_str().to_string())
}

fn compare_semver(v1: &str, v2: &str) -> std::cmp::Ordering {
    let parse = |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse().ok()).collect() };
    let p1 = parse(v1);
    let p2 = parse(v2);
    for i in 0..p1.len().max(p2.len()) {
        let a = p1.get(i).copied().unwrap_or(0);
        let b = p2.get(i).copied().unwrap_or(0);
        match a.cmp(&b) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

/// Try to fetch the latest Antigravity version from remote.
fn try_fetch_remote_version() -> Option<String> {
    const VERSION_URL: &str = "https://antigravity-auto-updater-974169037036.us-central1.run.app";

    let (tx, rx) = std::sync::mpsc::channel::<Option<String>>();

    std::thread::spawn(move || {
        let result = (|| -> Option<String> {
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .ok()?;

            let resp = client.get(VERSION_URL).send().ok()?;
            let text: String = resp.text().ok()?;
            parse_version(&text)
        })();
        let _ = tx.send(result);
    });

    rx.recv_timeout(std::time::Duration::from_secs(6))
        .unwrap_or(None)
}

fn resolve_best_version() -> String {
    let mut best = KNOWN_STABLE_VERSION.to_string();

    if let Some(remote_v) = try_fetch_remote_version() {
        if compare_semver(&remote_v, &best) > std::cmp::Ordering::Equal {
            best = remote_v;
        }
    }

    best
}

/// Current resolved version
pub static CURRENT_VERSION: LazyLock<String> = LazyLock::new(resolve_best_version);

/// Native OAuth User-Agent
pub static NATIVE_OAUTH_USER_AGENT: LazyLock<String> =
    LazyLock::new(|| format!("vscode/1.X.X (Antigravity/{})", CURRENT_VERSION.as_str()));

/// Global Session ID
pub static SESSION_ID: LazyLock<String> = LazyLock::new(|| uuid::Uuid::new_v4().to_string());

/// Full User-Agent string
pub static USER_AGENT: LazyLock<String> = LazyLock::new(|| {
    let version = CURRENT_VERSION.as_str();

    let platform_info = match std::env::consts::OS {
        "macos" => "Macintosh; Intel Mac OS X 10_15_7",
        "windows" => "Windows NT 10.0; Win64; x64",
        _ => "X11; Linux x86_64",
    };

    format!(
        "Antigravity/{} ({}) Chrome/{} Electron/{}",
        version, platform_info, KNOWN_STABLE_CHROME, KNOWN_STABLE_ELECTRON
    )
});

pub fn get_user_agent() -> String {
    USER_AGENT.clone()
}

pub fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
