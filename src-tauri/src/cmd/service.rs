use reqwest;
use tauri::State;

use crate::core::service::{
    check_service_status as check_service_status_impl, install_service as install_service_impl,
    start_service as start_service_impl, uninstall_service as uninstall_service_impl,
};
use crate::object::structs::AppState;
use crate::utils::api::{get_api_key, get_server_port};

#[tauri::command]
pub async fn check_service_status() -> Result<String, String> {
    check_service_status_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_service() -> Result<bool, String> {
    install_service_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn uninstall_service() -> Result<bool, String> {
    uninstall_service_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_service(_state: State<'_, AppState>) -> Result<bool, String> {
    let api_key = get_api_key();
    let port = get_server_port();
    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://127.0.0.1:{port}/api/v1/service/stop"))
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {e}"))?;
    if response.status().is_success() {
        Ok(true)
    } else {
        Err(format!("Failed to stop service: {}", response.status()))
    }
}

#[tauri::command]
pub async fn start_service() -> Result<bool, String> {
    start_service_impl().await.map_err(|e| e.to_string())
}
