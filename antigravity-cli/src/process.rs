use std::process::Command;
use std::thread;
use std::time::Duration;
use sysinfo::System;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Check if a process is an Antigravity main process (not a thread/worker).
/// Cross-platform logic with special handling for macOS .app bundles.
fn is_main_antigravity_process(name: &str, exe_path: &str, args_str: &str, target: &str) -> bool {
    let name_lower = name.to_lowercase();
    let exe_path_lower = exe_path.to_lowercase();

    // Exclude extension/config directory processes
    if exe_path_lower.contains("/.antigravity-ide/") || exe_path_lower.contains("/.antigravity/") {
        return false;
    }

    // Exclude well-known non-Antigravity binaries
    let binary_name = exe_path.rsplit('/').next().unwrap_or("").to_lowercase();
    if matches!(binary_name.as_str(), "java" | "node" | "rust-analyzer" | "gopls" | "python" | "python3" | "git") {
        return false;
    }

    // Exclude helper sub-processes (crashpad, language_server, renderer, gpu, plugin, etc.)
    if name_lower.contains("crashpad")
        || name_lower.contains("language_server")
        || name_lower.contains("helper")
        || name_lower.contains("plugin")
        || name_lower.contains("renderer")
        || name_lower.contains("gpu")
        || name_lower.contains("utility")
        || name_lower.contains("audio")
        || name_lower.contains("sandbox")
        || exe_path_lower.contains("crashpad")
        || exe_path_lower.contains("language_server")
    {
        return false;
    }

    // Exclude Electron sub-process types (identified by --type= arg)
    if args_str.contains("--type=") {
        return false;
    }

    // Platform-specific logic
    if target == "ide" {
        // IDE target: Match "Antigravity IDE"
        #[cfg(target_os = "macos")]
        {
            // On macOS: Main process is often "Electron", check if inside Antigravity IDE.app bundle
            if exe_path_lower.contains("antigravity ide.app") || exe_path_lower.contains("antigravity-ide.app") {
                // Additional check: must not be a helper explicitly (already filtered above)
                return !name_lower.contains("helper") && !name_lower.contains("gpu")
                    && !name_lower.contains("renderer") && !name_lower.contains("plugin");
            }
            false
        }

        #[cfg(not(target_os = "macos"))]
        {
            // On Linux/Windows: Check process name and path
            (name_lower.contains("antigravity-ide") || name_lower.contains("antigravity ide"))
                && (exe_path_lower.contains("antigravity-ide") || exe_path_lower.contains("antigravity ide"))
        }
    } else {
        // Classic/Client target: Match "Antigravity" but NOT "IDE"
        #[cfg(target_os = "macos")]
        {
            // On macOS: Check if inside Antigravity.app bundle (not IDE)
            if exe_path_lower.contains("antigravity.app") && !exe_path_lower.contains("antigravity ide.app") && !exe_path_lower.contains("antigravity-ide.app") {
                // Additional check: must not be a helper explicitly (already filtered above)
                return !name_lower.contains("helper") && !name_lower.contains("gpu")
                    && !name_lower.contains("renderer") && !name_lower.contains("plugin");
            }
            false
        }

        #[cfg(not(target_os = "macos"))]
        {
            // On Linux/Windows: Check process name and path
            (name_lower.contains("antigravity") && !name_lower.contains("antigravity-ide") && !name_lower.contains("antigravity ide"))
                && (exe_path_lower.contains("antigravity") && !exe_path_lower.contains("antigravity-ide") && !exe_path_lower.contains("antigravity ide"))
        }
    }
}

/// Check if a process is our own CLI tool (to exclude from matching)
fn is_own_cli_process(name: &str, exe_path: &str, args_str: &str) -> bool {
    name.contains("antigravity-cli")
        || exe_path.contains("antigravity-cli")
        || name == "cargo"
        || args_str.contains("antigravity-cli")
}

pub fn is_antigravity_running(target: &str) -> bool {
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

        if is_own_cli_process(&name, &exe_path, &args_str) {
            continue;
        }

        if is_main_antigravity_process(&name, &exe_path, &args_str, target) {
            #[cfg(target_os = "macos")]
            println!("[DEBUG] Found Antigravity ({}) process: PID={}, Name='{}', Exe='{}'",
                     target, pid.as_u32(), name, exe_path);
            return true;
        }
    }

    false
}

/// Find PIDs of main Antigravity processes (only main processes, not thread workers)
fn get_main_pids(target: &str) -> Vec<u32> {
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);
    let mut pids = Vec::new();
    let current_pid = std::process::id();

    #[cfg(target_os = "linux")]
    let family_pids = get_self_family_pids(&system);

    #[cfg(target_os = "macos")]
    println!("[DEBUG] Scanning for '{}' processes on macOS...", target);

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

        if is_own_cli_process(&name, &exe_path, &args_str) {
            continue;
        }

        #[cfg(target_os = "linux")]
        {
            if family_pids.contains(&pid_u32) {
                continue;
            }
        }

        if is_main_antigravity_process(&name, &exe_path, &args_str, target) {
            #[cfg(target_os = "macos")]
            {
                println!("[DEBUG] ✓ Matched PID={}: name='{}', exe='{}'",
                         pid_u32, name, exe_path);
            }
            pids.push(pid_u32);
        }
    }

    if !pids.is_empty() {
        println!("Found {} main Antigravity process(es):", pids.len());
        for &pid_u32 in &pids {
            let pid = sysinfo::Pid::from_u32(pid_u32);
            if let Some(process) = system.process(pid) {
                let name = process.name().to_string_lossy();
                let exe = process.exe().and_then(|p| p.to_str()).unwrap_or("(unknown)");
                println!("  PID={}, Name={}, Exe={}", pid_u32, name, exe);
            }
        }
    } else {
        #[cfg(target_os = "macos")]
        println!("[DEBUG] No matching processes found for target '{}'", target);
    }

    pids
}

#[cfg(target_os = "linux")]
fn get_self_family_pids(system: &sysinfo::System) -> std::collections::HashSet<u32> {
    let current_pid = std::process::id();
    let mut family_pids = std::collections::HashSet::new();
    family_pids.insert(current_pid);

    let mut next_pid = current_pid;
    for _ in 0..10 {
        let pid_val = sysinfo::Pid::from_u32(next_pid);
        if let Some(process) = system.process(pid_val) {
            if let Some(parent) = process.parent() {
                let parent_id = parent.as_u32();
                if !family_pids.insert(parent_id) {
                    break;
                }
                next_pid = parent_id;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let mut adj: std::collections::HashMap<u32, Vec<u32>> = std::collections::HashMap::new();
    for (pid, process) in system.processes() {
        if let Some(parent) = process.parent() {
            adj.entry(parent.as_u32()).or_default().push(pid.as_u32());
        }
    }

    let mut queue = std::collections::VecDeque::new();
    queue.push_back(current_pid);
    while let Some(pid) = queue.pop_front() {
        if let Some(children) = adj.get(&pid) {
            for &child in children {
                if family_pids.insert(child) {
                    queue.push_back(child);
                }
            }
        }
    }

    family_pids
}

pub fn close_antigravity(timeout_secs: u64, target: &str) -> Result<(), String> {
    let label = if target == "ide" { "Antigravity IDE" } else { "Antigravity" };
    println!("Closing {}...", label);

    let pids = get_main_pids(target);
    if pids.is_empty() {
        println!("{} is not running.", label);
        return Ok(());
    }

    // Phase 1: SIGTERM to main processes
    #[cfg(target_os = "windows")]
    {
        for pid in &pids {
            let _ = Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .creation_flags(0x08000000)
                .output();
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        for pid in &pids {
            println!("Sending SIGTERM to PID={}", pid);
            let _ = Command::new("kill")
                .args(["-15", &pid.to_string()])
                .output();
        }
    }

    // Wait for graceful exit (70% of timeout)
    let graceful_timeout = (timeout_secs * 7) / 10;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(graceful_timeout) {
        if !is_antigravity_running(target) {
            println!("{} closed successfully (graceful).", label);
            return Ok(());
        }
        thread::sleep(Duration::from_millis(500));
    }

    // Phase 2: SIGKILL remaining main processes
    #[cfg(not(target_os = "windows"))]
    {
        let remaining = get_main_pids(target);
        if !remaining.is_empty() {
            println!("Graceful exit timeout. Force killing {} remaining main process(es)...", remaining.len());
            for pid in &remaining {
                println!("Sending SIGKILL to PID={}", pid);
                let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
            }
        }
    }

    // Wait for cleanup (remaining 30% of timeout)
    let kill_timeout = timeout_secs - graceful_timeout;
    let start2 = std::time::Instant::now();
    while start2.elapsed() < Duration::from_secs(kill_timeout.max(3)) {
        if !is_antigravity_running(target) {
            println!("{} closed successfully (forced).", label);
            return Ok(());
        }
        thread::sleep(Duration::from_millis(500));
    }

    // Final check
    if is_antigravity_running(target) {
        return Err(format!("Unable to close {} process, please close manually and retry", label));
    }

    println!("{} closed successfully.", label);
    Ok(())
}

fn get_antigravity_executable_path(target: &str) -> Option<std::path::PathBuf> {
    let folder_name = if target == "ide" {
        "Antigravity IDE"
    } else {
        "Antigravity"
    };

    #[cfg(target_os = "macos")]
    {
        let path = std::path::PathBuf::from(format!("/Applications/{}.app", folder_name));
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
            possible_paths.push(
                std::path::PathBuf::from(&local)
                    .join("Programs")
                    .join(folder_name)
                    .join(format!("{}.exe", folder_name)),
            );
        }
        possible_paths.push(
            std::path::PathBuf::from(&program_files)
                .join(folder_name)
                .join(format!("{}.exe", folder_name)),
        );
        possible_paths.push(
            std::path::PathBuf::from(&program_files_x86)
                .join(folder_name)
                .join(format!("{}.exe", folder_name)),
        );

        for path in possible_paths {
            if path.exists() {
                return Some(path);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let exe_name = if target == "ide" {
            "antigravity-ide"
        } else {
            "antigravity"
        };

        if let Some(home) = dirs::home_dir() {
            let user_local = home.join(format!(".local/bin/{}", exe_name));
            if user_local.exists() {
                return Some(user_local);
            }
        }

        let possible_paths = vec![
            std::path::PathBuf::from(format!("/usr/bin/{}", exe_name)),
            std::path::PathBuf::from(format!("/opt/{}/{}", folder_name, exe_name)),
            std::path::PathBuf::from(format!("/opt/{}/{}", exe_name, exe_name)),
            std::path::PathBuf::from(format!("/usr/share/{}/{}", folder_name, exe_name)),
        ];

        for path in possible_paths {
            if path.exists() {
                return Some(path);
            }
        }
    }

    None
}

pub fn start_antigravity(target: &str) -> Result<(), String> {
    let label = if target == "ide" { "Antigravity IDE" } else { "Antigravity" };
    println!("Starting {}...", label);

    #[cfg(target_os = "macos")]
    {
        let app_name = if target == "ide" {
            "Antigravity IDE"
        } else {
            "Antigravity"
        };
        let output = Command::new("open")
            .args(["-a", app_name])
            .output()
            .map_err(|e| format!("Unable to execute open command: {}", e))?;

        if !output.status.success() {
            return Err(format!("Startup failed: {}", String::from_utf8_lossy(&output.stderr)));
        }

        println!("Sent start command to {}. Waiting for app to launch...", label);

        // Wait up to 8 seconds for the app process to appear
        let mut attempts = 0;
        let max_attempts = 40; // 40 * 200ms = 8 seconds

        while attempts < max_attempts {
            thread::sleep(Duration::from_millis(200));
            if is_antigravity_running(target) {
                println!("{} started successfully (verified running).", label);
                return Ok(());
            }
            attempts += 1;
        }

        println!("{} launch command executed, but process detection timeout - app may still be starting.", label);
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let path = get_antigravity_executable_path(target)
            .ok_or(format!("Cannot find {} executable", label))?;
        let mut cmd = Command::new(&path);
        cmd.creation_flags(0x08000000);
        cmd.spawn().map_err(|e| format!("Startup failed: {}", e))?;
        println!("{} started successfully.", label);
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        let path = get_antigravity_executable_path(target)
            .ok_or(format!("Cannot find {} executable", label))?;
        println!("Found executable: {:?}", path);
        let _ = Command::new(&path)
            .spawn()
            .map_err(|e| format!("Startup failed: {}", e))?;
        println!("{} started successfully.", label);
        return Ok(());
    }
}
