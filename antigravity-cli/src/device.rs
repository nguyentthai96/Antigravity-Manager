use crate::models::DeviceProfile;
use rand::{distributions::Alphanumeric, Rng};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use rusqlite::Connection;
use crate::db::get_db_path;

/// Resolve folder name based on target variant
fn get_folder_name(target: &str) -> &str {
    if target == "ide" {
        "Antigravity IDE"
    } else {
        "Antigravity"
    }
}

pub fn get_storage_path(target: &str) -> Result<PathBuf, String> {
    let folder_name = get_folder_name(target);

    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("failed_to_get_home_dir")?;
        Ok(home.join(format!(
            "Library/Application Support/{}/User/globalStorage/storage.json",
            folder_name
        )))
    }
    #[cfg(target_os = "windows")]
    {
        let appdata =
            std::env::var("APPDATA").map_err(|_| "failed_to_get_appdata_env".to_string())?;
        Ok(PathBuf::from(appdata).join(format!(
            "{}\\User\\globalStorage\\storage.json",
            folder_name
        )))
    }
    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().ok_or("failed_to_get_home_dir")?;
        Ok(home.join(format!(
            ".config/{}/User/globalStorage/storage.json",
            folder_name
        )))
    }
}

pub fn write_profile(storage_path: &Path, profile: &DeviceProfile) -> Result<(), String> {
    if !storage_path.exists() {
        // If it doesn't exist, create parent dirs and an empty one with telemetry object
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed_to_create_storage_dir: {}", e))?;
        }
        let initial_json = serde_json::json!({
            "telemetry": {}
        });
        fs::write(storage_path, serde_json::to_string_pretty(&initial_json).unwrap())
            .map_err(|e| format!("failed_to_create_storage_json: {}", e))?;
    }

    let content =
        fs::read_to_string(storage_path).map_err(|e| format!("read_failed: {}", e))?;
    let mut json: Value =
        serde_json::from_str(&content).map_err(|e| format!("parse_failed: {}", e))?;

    if !json.get("telemetry").map_or(false, |v| v.is_object()) {
        if json.as_object_mut().is_some() {
            json["telemetry"] = serde_json::json!({});
        } else {
            return Err("json_top_level_not_object".to_string());
        }
    }

    if let Some(telemetry) = json.get_mut("telemetry").and_then(|v| v.as_object_mut()) {
        telemetry.insert(
            "machineId".to_string(),
            Value::String(profile.machine_id.clone()),
        );
        telemetry.insert(
            "macMachineId".to_string(),
            Value::String(profile.mac_machine_id.clone()),
        );
        telemetry.insert(
            "devDeviceId".to_string(),
            Value::String(profile.dev_device_id.clone()),
        );
        telemetry.insert("sqmId".to_string(), Value::String(profile.sqm_id.clone()));
    }

    if let Some(map) = json.as_object_mut() {
        map.insert(
            "telemetry.machineId".to_string(),
            Value::String(profile.machine_id.clone()),
        );
        map.insert(
            "telemetry.macMachineId".to_string(),
            Value::String(profile.mac_machine_id.clone()),
        );
        map.insert(
            "telemetry.devDeviceId".to_string(),
            Value::String(profile.dev_device_id.clone()),
        );
        map.insert(
            "telemetry.sqmId".to_string(),
            Value::String(profile.sqm_id.clone()),
        );
        map.insert(
            "storage.serviceMachineId".to_string(),
            Value::String(profile.dev_device_id.clone()),
        );
    }

    let updated = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("serialize_failed: {}", e))?;
    fs::write(storage_path, updated).map_err(|e| format!("write_failed ({:?}): {}", storage_path, e))?;
    println!("Device profile written to {:?}", storage_path);

    // Sync serviceMachineId to state.vscdb using the same target
    // We derive target from the storage_path folder name
    let target_from_path = if storage_path.to_string_lossy().contains("Antigravity IDE") {
        "ide"
    } else {
        "classic"
    };
    let _ = sync_state_service_machine_id_value(&profile.dev_device_id, target_from_path);
    Ok(())
}

fn sync_state_service_machine_id_value(service_id: &str, target: &str) -> Result<(), String> {
    let db_path = get_db_path(target)?;
    if !db_path.exists() {
        return Ok(());
    }

    let conn = Connection::open(&db_path).map_err(|e| format!("db_open_failed: {}", e))?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ItemTable (key TEXT PRIMARY KEY, value TEXT);",
        [],
    )
    .map_err(|e| format!("failed_to_create_item_table: {}", e))?;
    conn.execute(
        "INSERT OR REPLACE INTO ItemTable (key, value) VALUES ('storage.serviceMachineId', ?1);",
        [service_id],
    )
    .map_err(|e| format!("failed_to_write_to_db: {}", e))?;
    Ok(())
}

pub fn generate_profile() -> DeviceProfile {
    DeviceProfile {
        machine_id: format!("auth0|user_{}", random_hex(32)),
        mac_machine_id: new_standard_machine_id(),
        dev_device_id: Uuid::new_v4().to_string(),
        sqm_id: format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase()),
    }
}

fn random_hex(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect::<String>()
        .to_lowercase()
}

fn new_standard_machine_id() -> String {
    let mut rng = rand::thread_rng();
    let mut id = String::with_capacity(36);
    for ch in "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".chars() {
        if ch == '-' || ch == '4' {
            id.push(ch);
        } else if ch == 'x' {
            id.push_str(&format!("{:x}", rng.gen_range(0..16)));
        } else if ch == 'y' {
            id.push_str(&format!("{:x}", rng.gen_range(8..12)));
        }
    }
    id
}
