use std::str::FromStr;

use reqwest;
use tauri::State;

use crate::object::structs::AppState;
use crate::utils::api::{ListProcessResponse, ProcessStatus, get_api_key, get_server_port};

#[tauri::command]
pub async fn get_process_list(_state: State<'_, AppState>) -> Result<Vec<ProcessStatus>, String> {
    let api_key = get_api_key();
    let port = get_server_port();
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://127.0.0.1:{port}/api/v1/processes"))
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {e}"))?;
    if response.status().is_success() {
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response text: {e}"))?;
        let process_list = match serde_json::from_str::<ListProcessResponse>(&response_text) {
            Ok(process_list) => process_list,
            Err(e) => {
                return Err(format!(
                    "Failed to parse response: {e}, response: {response_text}"
                ));
            }
        };
        Ok(process_list.data)
    } else {
        Err(format!("Failed to get process list: {}", response.status()))
    }
}

#[tauri::command]
pub async fn start_process(id: String, _state: State<'_, AppState>) -> Result<bool, String> {
    let api_key = get_api_key();
    let port = get_server_port();
    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "http://127.0.0.1:{port}/api/v1/processes/{id}/start"
        ))
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {e}"))?;
    if response.status().is_success() {
        Ok(true)
    } else {
        Err(format!("Failed to start process: {}", response.status()))
    }
}

#[tauri::command]
pub async fn stop_process(id: String, _state: State<'_, AppState>) -> Result<bool, String> {
    let api_key = get_api_key();
    let port = get_server_port();
    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "http://127.0.0.1:{port}/api/v1/processes/{id}/stop"
        ))
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {e}"))?;
    if response.status().is_success() {
        Ok(true)
    } else {
        Err(format!("Failed to stop process: {}", response.status()))
    }
}

#[tauri::command]
pub async fn restart_process(id: String, _state: State<'_, AppState>) -> Result<bool, String> {
    let api_key = get_api_key();
    let port = get_server_port();
    let client = reqwest::Client::new();
    let stop_response = client
        .post(format!(
            "http://127.0.0.1:{port}/api/v1/processes/{id}/stop"
        ))
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {e}"))?;
    if stop_response.status().is_success() {
        let start_response = client
            .post(
                url::Url::from_str(&format!(
                    "http://127.0.0.1:{port}/api/v1/processes/{id}/start"
                ))
                .unwrap(),
            )
            .header("Authorization", format!("Bearer {api_key}"))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {e}"))?;
        if start_response.status().is_success() {
            Ok(true)
        } else {
            Err(format!(
                "Failed to start OpenList Core process: {}",
                start_response.status()
            ))
        }
    } else {
        Err(format!(
            "Failed to stop OpenList Core process: {}",
            stop_response.status()
        ))
    }
}

#[tauri::command]
pub async fn update_process(
    id: String,
    update_config: serde_json::Value,
    _state: State<'_, AppState>,
) -> Result<bool, String> {
    let api_key = get_api_key();
    let port = get_server_port();
    let client = reqwest::Client::new();
    let response = client
        .put(format!("http://127.0.0.1:{port}/api/v1/processes/{id}"))
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&update_config)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {e}"))?;
    if response.status().is_success() {
        Ok(true)
    } else {
        Err(format!("Failed to update process: {}", response.status()))
    }
}
