use std::process::Command;
use std::thread;
use std::time::Duration;
use sysinfo::System;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

fn get_current_exe_path() -> Option<std::path::PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.canonicalize().ok())
}

pub fn is_antigravity_running() -> bool {
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);

    let current_pid = std::process::id();

    for (pid, process) in system.processes() {
        if pid.as_u32() == current_pid {
            continue;
        }

        let name = process.name().to_string_lossy().to_lowercase();
        let exe_path = process
            .exe()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_lowercase();

        let args = process.cmd();
        let args_str = args
            .iter()
            .map(|arg| arg.to_string_lossy().to_lowercase())
            .collect::<Vec<String>>()
            .join(" ");

        // Prevent matching our own CLI process
        if name.contains("antigravity-cli") || exe_path.contains("antigravity-cli") || name == "cargo" || args_str.contains("antigravity-cli") {
            continue;
        }

        let is_helper = args_str.contains("--type=")
            || name.contains("helper")
            || name.contains("plugin")
            || name.contains("renderer")
            || name.contains("gpu")
            || name.contains("crashpad")
            || name.contains("utility")
            || name.contains("audio")
            || name.contains("sandbox")
            || exe_path.contains("crashpad");

        #[cfg(target_os = "macos")]
        if exe_path.contains("antigravity.app") && !is_helper {
            return true;
        }

        #[cfg(target_os = "windows")]
        if name == "antigravity.exe" && !is_helper {
            return true;
        }

        #[cfg(target_os = "linux")]
        if (name.contains("antigravity") || exe_path.contains("/antigravity"))
            && !name.contains("tools")
            && !is_helper
        {
            return true;
        }
    }

    false
}

fn get_antigravity_pids() -> Vec<u32> {
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);
    let mut pids = Vec::new();
    let current_pid = std::process::id();

    for (pid, process) in system.processes() {
        let pid_u32 = pid.as_u32();
        if pid_u32 == current_pid {
            continue;
        }

        let name = process.name().to_string_lossy().to_lowercase();
        let exe_path = process
            .exe()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_lowercase();

        let args = process.cmd();
        let args_str = args
            .iter()
            .map(|arg| arg.to_string_lossy().to_lowercase())
            .collect::<Vec<String>>()
            .join(" ");

        // Prevent matching our own CLI process
        if name.contains("antigravity-cli") || exe_path.contains("antigravity-cli") || name == "cargo" || args_str.contains("antigravity-cli") {
            continue;
        }

        let is_helper = args_str.contains("--type=")
            || name.contains("helper")
            || name.contains("plugin")
            || name.contains("renderer")
            || name.contains("gpu")
            || name.contains("crashpad")
            || name.contains("utility")
            || name.contains("audio")
            || name.contains("sandbox")
            || exe_path.contains("crashpad");

        #[cfg(target_os = "macos")]
        if exe_path.contains("antigravity.app") {
            pids.push(pid_u32);
        }

        #[cfg(target_os = "windows")]
        if name == "antigravity.exe" {
            pids.push(pid_u32);
        }

        #[cfg(target_os = "linux")]
        if (name == "antigravity" || exe_path.contains("/antigravity")) && !name.contains("tools") {
            println!("Matched process: PID={}, Name={}, ExePath={}", pid_u32, name, exe_path);
            pids.push(pid_u32);
        }
    }
    pids
}

pub fn close_antigravity(timeout_secs: u64) -> Result<(), String> {
    println!("Closing Antigravity...");
    let pids = get_antigravity_pids();
    
    if pids.is_empty() {
        println!("Antigravity is not running.");
        return Ok(());
    }
    
    println!("Found PIDs to kill: {:?}", pids);

    #[cfg(target_os = "windows")]
    {
        for pid in pids {
            let _ = Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .creation_flags(0x08000000)
                .output();
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        for pid in &pids {
            println!("Sending SIGTERM to {}", pid);
            let _ = Command::new("kill")
                .args(["-15", &pid.to_string()])
                .output();
        }
    }

    let graceful_timeout = (timeout_secs * 7) / 10;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(graceful_timeout) {
        if !is_antigravity_running() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(500));
    }

    #[cfg(not(target_os = "windows"))]
    {
        let remaining_pids = get_antigravity_pids();
        for pid in &remaining_pids {
            let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
        }
    }
    
    thread::sleep(Duration::from_secs(1));
    if is_antigravity_running() {
        return Err("Unable to close Antigravity process".to_string());
    }

    Ok(())
}

fn get_antigravity_executable_path() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let path = std::path::PathBuf::from("/Applications/Antigravity.app");
        if path.exists() {
            return Some(path);
        }
    }

    #[cfg(target_os = "windows")]
    {
        let local_appdata = std::env::var("LOCALAPPDATA").ok();
        let program_files = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
        let program_files_x86 = std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());

        let mut possible_paths = Vec::new();
        if let Some(local) = local_appdata {
            possible_paths.push(std::path::PathBuf::from(&local).join("Programs").join("Antigravity").join("Antigravity.exe"));
        }
        possible_paths.push(std::path::PathBuf::from(&program_files).join("Antigravity").join("Antigravity.exe"));
        possible_paths.push(std::path::PathBuf::from(&program_files_x86).join("Antigravity").join("Antigravity.exe"));

        for path in possible_paths {
            if path.exists() {
                return Some(path);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let possible_paths = vec![
            std::path::PathBuf::from("/usr/bin/antigravity"),
            std::path::PathBuf::from("/opt/Antigravity/antigravity"),
        ];

        if let Some(home) = dirs::home_dir() {
            let user_local = home.join(".local/bin/antigravity");
            if user_local.exists() {
                return Some(user_local);
            }
        }

        for path in possible_paths {
            if path.exists() {
                return Some(path);
            }
        }
    }

    None
}

pub fn start_antigravity() -> Result<(), String> {
    println!("Starting Antigravity...");

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("open")
            .args(["-a", "Antigravity"])
            .output()
            .map_err(|e| format!("Unable to execute open command: {}", e))?;

        if !output.status.success() {
            return Err(format!("Startup failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
    }

    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("cmd");
        cmd.creation_flags(0x08000000);
        cmd.args(["/C", "start", "antigravity://"]);
        let result = cmd.spawn();
        if result.is_err() {
            return Err("Startup failed".to_string());
        }
    }

    #[cfg(target_os = "linux")]
    {
        let path = get_antigravity_executable_path().ok_or("Cannot find Antigravity executable")?;
        let _ = Command::new(path).spawn().map_err(|e| format!("Startup failed: {}", e))?;
    }

    Ok(())
}
