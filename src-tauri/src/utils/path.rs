use std::path::PathBuf;
use std::{env, fs};

pub static APP_ID: &str = "io.github.openlistteam.openlist.desktop";

fn get_app_dir() -> Result<PathBuf, String> {
    let app_dir = env::current_exe()
        .map_err(|e| format!("Failed to get current exe path: {e}"))?
        .parent()
        .ok_or("Failed to get parent directory")?
        .to_path_buf();
    if !app_dir.exists() {
        return Err(format!("Application directory does not exist: {app_dir:?}"));
    }

    Ok(app_dir)
}

pub fn get_openlist_binary_path() -> Result<PathBuf, String> {
    let app_dir = get_app_dir()?;

    let binary_name = if cfg!(target_os = "windows") {
        "openlist.exe"
    } else {
        "openlist"
    };
    let binary_path = app_dir.join(binary_name);

    if !binary_path.exists() {
        return Err(format!(
            "OpenList service binary not found at: {binary_path:?}"
        ));
    }

    Ok(binary_path)
}

pub fn get_rclone_binary_path() -> Result<PathBuf, String> {
    let app_dir = get_app_dir()?;

    let binary_name = if cfg!(target_os = "windows") {
        "rclone.exe"
    } else {
        "rclone"
    };
    let binary_path = app_dir.join(binary_name);

    if !binary_path.exists() {
        return Err(format!(
            "Rclone service binary not found at: {binary_path:?}"
        ));
    }

    Ok(binary_path)
}

pub fn get_app_config_dir() -> Result<PathBuf, String> {
    let app_dir = get_app_dir()?;
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    Ok(app_dir)
}

pub fn app_config_file_path() -> Result<PathBuf, String> {
    Ok(get_app_config_dir()?.join("settings.json"))
}

pub fn get_app_logs_dir() -> Result<PathBuf, String> {
    let app_dir = get_app_dir()?;
    let logs_dir = app_dir.join("logs");
    fs::create_dir_all(&logs_dir).map_err(|e| e.to_string())?;
    Ok(logs_dir)
}
