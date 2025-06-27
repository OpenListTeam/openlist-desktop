use regex::Regex;
use std::env;

#[tauri::command]
pub async fn get_admin_password() -> Result<String, String> {
    let app_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let logs_dir = app_dir.join("logs/process_openlist_core.log");

    let logs_content =
        std::fs::read_to_string(logs_dir).map_err(|e| format!("Failed to read log file: {e}"))?;

    let re = Regex::new(r"Successfully created the admin user and the initial password is: (\w+)")
        .map_err(|e| format!("Failed to create regex: {e}"))?;

    let mut last_password = None;
    for line in logs_content.lines() {
        if let Some(captures) = re.captures(line) {
            if let Some(password) = captures.get(1) {
                last_password = Some(password.as_str().to_string());
            }
        }
    }

    last_password.ok_or("No admin password found in logs".to_string())
}

#[tauri::command]
pub async fn get_logs(source: Option<String>) -> Result<Vec<String>, String> {
    match source.as_deref() {
        Some("openlist") => {
            let app_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
            let logs_dir = app_dir.join("data/log/log.log");
            let logs = std::fs::read_to_string(logs_dir)
                .map_err(|e| e.to_string())?
                .lines()
                .map(|line| line.to_string())
                .collect();
            Ok(logs)
        }
        Some("app") => {
            let app_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
            let logs_dir = app_dir.join("logs/app.log");
            let logs = std::fs::read_to_string(logs_dir)
                .map_err(|e| e.to_string())?
                .lines()
                .map(|line| line.to_string())
                .collect();
            Ok(logs)
        }
        Some("rclone") => {
            let app_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
            let logs_dir = app_dir.join("logs/process_rclone.log");
            let logs = std::fs::read_to_string(logs_dir)
                .map_err(|e| e.to_string())?
                .lines()
                .map(|line| line.to_string())
                .collect();
            Ok(logs)
        }
        Some("openlist_core") => {
            let app_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
            let logs_dir = app_dir.join("logs/process_openlist_core.log");
            let logs = std::fs::read_to_string(logs_dir)
                .map_err(|e| e.to_string())?
                .lines()
                .map(|line| line.to_string())
                .collect();
            Ok(logs)
        }
        _ => Err("Invalid log source".to_string()),
    }
}

#[tauri::command]
pub async fn clear_logs() -> Result<bool, String> {
    Ok(true)
}
