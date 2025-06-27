use std::fs;
use tauri::State;

use crate::cmd::http_api::{get_process_list, start_process, stop_process};
use crate::conf::config::MergedSettings;
use crate::object::structs::AppState;
use crate::utils::path::app_config_file_path;

#[tauri::command]
pub async fn save_settings(
    settings: MergedSettings,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    state.update_settings(settings.clone());
    let settings_path = app_config_file_path().map_err(|e| e.to_string())?;
    let settings_json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(settings_path, settings_json).map_err(|e| e.to_string())?;

    log::info!("Settings saved successfully");
    Ok(true)
}

#[tauri::command]
pub async fn save_settings_with_update_port(
    settings: MergedSettings,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    save_settings(settings.clone(), state.clone()).await?;
    let app_dir = std::env::current_exe()
        .map_err(|e| format!("Failed to get current exe path: {e}"))?
        .parent()
        .ok_or("Failed to get parent directory")?
        .to_path_buf();
    let data_config_path = app_dir.join("data").join("config.json");
    if let Some(parent) = data_config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut config = if data_config_path.exists() {
        let content =
            std::fs::read_to_string(data_config_path.clone()).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())?
    } else {
        serde_json::json!({
            "scheme": {
                "http_port": settings.openlist.port,
            }
        })
    };
    if let Some(scheme) = config.get_mut("scheme") {
        if let Some(scheme_obj) = scheme.as_object_mut() {
            scheme_obj.insert(
                "http_port".to_string(),
                serde_json::Value::Number(serde_json::Number::from(settings.openlist.port)),
            );
        }
    } else {
        config["scheme"] = serde_json::json!({
            "http_port": settings.openlist.port
        });
    }
    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(data_config_path, content).map_err(|e| e.to_string())?;
    // Stop the OpenList core process
    let process_list = get_process_list(state.clone()).await?;
    if let Some(existing_process) = process_list
        .iter()
        .find(|p| p.config.name == "single_openlist_core_process")
    {
        match stop_process(existing_process.config.id.clone(), state.clone()).await {
            Ok(_) => log::info!("OpenList core process stopped successfully"),
            Err(e) => log::warn!("Failed to stop OpenList core process: {e}"),
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        match start_process(existing_process.config.id.clone(), state.clone()).await {
            Ok(_) => log::info!("OpenList core process started successfully with new port"),
            Err(e) => {
                log::error!("Failed to start OpenList core process: {e}");
                return Err(format!(
                    "Failed to restart OpenList core with new port: {e}"
                ));
            }
        }
    }

    log::info!("Settings saved and OpenList core restarted with new port successfully");
    Ok(true)
}

#[tauri::command]
pub async fn load_settings(state: State<'_, AppState>) -> Result<Option<MergedSettings>, String> {
    state.load_settings()?;
    Ok(state.get_settings())
}

#[tauri::command]
pub async fn reset_settings(state: State<'_, AppState>) -> Result<Option<MergedSettings>, String> {
    let default_settings = MergedSettings::default();
    state.update_settings(default_settings.clone());

    let settings_path = app_config_file_path().map_err(|e| e.to_string())?;
    let settings_json =
        serde_json::to_string_pretty(&default_settings).map_err(|e| e.to_string())?;
    fs::write(settings_path, settings_json).map_err(|e| e.to_string())?;

    log::info!("Settings reset to default");
    Ok(Some(default_settings))
}
