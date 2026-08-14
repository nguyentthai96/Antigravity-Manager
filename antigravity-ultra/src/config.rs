use std::path::PathBuf;

/// Get data directory for antigravity-ultra
pub fn get_data_dir() -> PathBuf {
    let dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".antigravity_ultra");

    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }

    // Set restrictive permissions on Linux/macOS
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
    }

    dir
}
