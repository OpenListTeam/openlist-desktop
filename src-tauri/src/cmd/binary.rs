use std::env;
use std::process::Command;

#[tauri::command]
pub async fn get_binary_version(binary_name: Option<String>) -> Result<String, String> {
    let app_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let binary_path = if cfg!(windows) {
        app_dir.join(format!(
            "{}.exe",
            binary_name.unwrap_or("openlist".to_string())
        ))
    } else {
        app_dir.join(binary_name.unwrap_or("openlist".to_string()))
    };
    let output = Command::new(binary_path)
        .arg("version")
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        let version_output = String::from_utf8_lossy(&output.stdout);
        let version_line = version_output
            .lines()
            .find(|line| line.starts_with("Version:") || line.starts_with("rclone"))
            .ok_or("Version not found in output")?;
        let version = version_line
            .split_whitespace()
            .nth(1)
            .ok_or("Failed to parse version")?;
        Ok(version.to_string())
    } else {
        Err("Failed to get OpenList binary version".to_string())
    }
}
