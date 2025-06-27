use crate::object::structs::{AppState, ServiceStatus};
use crate::utils::api::{CreateProcessResponse, ProcessConfig, get_api_key, get_server_port};
use crate::utils::path::{get_app_logs_dir, get_openlist_binary_path};
use reqwest;

use tauri::State;

use url::Url;

#[tauri::command]
pub async fn create_openlist_core_process(
    auto_start: bool,
    _state: State<'_, AppState>,
) -> Result<ProcessConfig, String> {
    let binary_path = get_openlist_binary_path()
        .map_err(|e| format!("Failed to get OpenList binary path: {e}"))?;
    let log_file_path =
        get_app_logs_dir().map_err(|e| format!("Failed to get app logs directory: {e}"))?;
    let log_file_path = log_file_path.join("process_openlist_core.log");

    let api_key = get_api_key();
    let port = get_server_port();

    let config = ProcessConfig {
        id: "openlist_core".into(),
        name: "single_openlist_core_process".into(),
        bin_path: binary_path.to_string_lossy().into_owned(),
        args: vec!["server".into()],
        log_file: log_file_path.to_string_lossy().into_owned(),
        working_dir: binary_path
            .parent()
            .map(|p| p.to_string_lossy().into_owned()),
        env_vars: None,
        auto_restart: true,
        auto_start,
        run_as_admin: false,
        created_at: 0,
        updated_at: 0,
    };
    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://127.0.0.1:{port}/api/v1/processes"))
        .json(&config)
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {e}"))?;
    if response.status().is_success() {
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response text: {e}"))?;
        let process_config = match serde_json::from_str::<CreateProcessResponse>(&response_text) {
            Ok(process_config) => process_config,
            Err(e) => {
                return Err(format!(
                    "Failed to parse response: {e}, response: {response_text}"
                ));
            }
        };
        Ok(process_config.data)
    } else {
        Err(format!(
            "Failed to create OpenList Core process: {}",
            response.status()
        ))
    }
}

#[tauri::command]
pub async fn get_openlist_core_status(state: State<'_, AppState>) -> Result<ServiceStatus, String> {
    let app_settings = state
        .app_settings
        .read()
        .clone()
        .ok_or("Failed to read app settings")?;
    let openlist_config = app_settings.openlist;
    let protocol = if openlist_config.ssl_enabled {
        "https"
    } else {
        "http"
    };

    let health_check_url = format!("{}://localhost:{}", protocol, openlist_config.port);

    let url =
        Url::parse(&health_check_url).map_err(|e| format!("Invalid health check URL: {e}"))?;
    let port = url.port_or_known_default();

    let health_url = format!("{health_check_url}/ping");
    let local_pid = None;

    match reqwest::get(&health_url).await {
        Ok(response) => {
            let is_running = response.status().is_success();
            Ok(ServiceStatus {
                running: is_running,
                pid: local_pid,
                port,
            })
        }
        Err(_) => Ok(ServiceStatus {
            running: false,
            pid: local_pid,
            port,
        }),
    }
}
