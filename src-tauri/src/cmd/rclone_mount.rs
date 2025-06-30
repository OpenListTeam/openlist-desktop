use std::fs;
use std::path::Path;

use reqwest::Client;
use serde_json::json;
use tauri::State;

use super::http_api::get_process_list;
use super::rclone_core::{RCLONE_API_BASE, RCLONE_AUTH};
use crate::conf::rclone::{RcloneCreateRemoteRequest, RcloneMountRequest, RcloneWebdavConfig};
use crate::object::structs::{
    AppState, RcloneMountInfo, RcloneMountListResponse, RcloneRemoteListResponse,
};
use crate::utils::api::{CreateProcessResponse, ProcessConfig, get_api_key, get_server_port};
use crate::utils::path::{get_app_logs_dir, get_rclone_binary_path};

#[tauri::command]
pub async fn rclone_list_config(
    remote_type: String,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = Client::new();
    let response = client
        .post(format!("{RCLONE_API_BASE}/config/dump"))
        .header("Authorization", RCLONE_AUTH)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {e}"))?;
    if response.status().is_success() {
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response text: {e}"))?;
        let json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse JSON: {e}"))?;
        let remotes = if remote_type.is_empty() {
            json.clone()
        } else if let Some(obj) = json.as_object() {
            let mut filtered_map = serde_json::Map::new();
            for (remote_name, remote_config) in obj {
                if let Some(config_obj) = remote_config.as_object()
                    && let Some(remote_type_value) = config_obj.get("type")
                    && let Some(type_str) = remote_type_value.as_str()
                    && type_str == remote_type
                {
                    filtered_map.insert(remote_name.clone(), remote_config.clone());
                }
            }
            serde_json::Value::Object(filtered_map)
        } else {
            serde_json::Value::Object(serde_json::Map::new())
        };

        Ok(remotes)
    } else {
        Err(format!(
            "Failed to list Rclone config: {}",
            response.status()
        ))
    }
}

#[tauri::command]
pub async fn rclone_list_remotes() -> Result<Vec<String>, String> {
    let client = Client::new();

    let response = client
        .post(format!("{RCLONE_API_BASE}/config/listremotes"))
        .header("Authorization", RCLONE_AUTH)
        .send()
        .await
        .map_err(|e| format!("Failed to list remotes: {e}"))?;

    if response.status().is_success() {
        let remote_list: RcloneRemoteListResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse remote list response: {e}"))?;
        Ok(remote_list.remotes)
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Failed to list remotes: {error_text}"))
    }
}

#[tauri::command]
pub async fn rclone_list_mounts() -> Result<RcloneMountListResponse, String> {
    let client = Client::new();

    let response = client
        .post(format!("{RCLONE_API_BASE}/mount/listmounts"))
        .header("Authorization", RCLONE_AUTH)
        .send()
        .await
        .map_err(|e| format!("Failed to list mounts: {e}"))?;

    if response.status().is_success() {
        let mount_list: RcloneMountListResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse mount list response: {e}"))?;
        Ok(mount_list)
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Failed to list mounts: {error_text}"))
    }
}

#[tauri::command]
pub async fn rclone_create_remote(
    name: String,
    r#type: String,
    config: RcloneWebdavConfig,
    _state: State<'_, AppState>,
) -> Result<bool, String> {
    let client = Client::new();

    let create_request = RcloneCreateRemoteRequest {
        name: name.clone(),
        r#type: r#type.clone(),
        parameters: crate::conf::rclone::RcloneWebdavConfig {
            url: config.url.clone(),
            vendor: config.vendor.clone(),
            user: config.user.clone(),
            pass: config.pass.clone(),
        },
    };

    let response = client
        .post(format!("{RCLONE_API_BASE}/config/create"))
        .header("Authorization", RCLONE_AUTH)
        .header("Content-Type", "application/json")
        .json(&create_request)
        .send()
        .await
        .map_err(|e| format!("Failed to create remote config: {e}"))?;

    if response.status().is_success() {
        Ok(true)
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Failed to create remote config: {error_text}"))
    }
}

#[tauri::command]
pub async fn rclone_update_remote(
    name: String,
    r#type: String,
    config: RcloneWebdavConfig,
    _state: State<'_, AppState>,
) -> Result<bool, String> {
    let client = Client::new();

    let response = client
        .post(format!("{RCLONE_API_BASE}/config/update"))
        .header("Authorization", RCLONE_AUTH)
        .header("Content-Type", "application/json")
        .json(&json!({ "name": name, "type": r#type, "parameters": config }))
        .send()
        .await
        .map_err(|e| format!("Failed to update remote config: {e}"))?;

    if response.status().is_success() {
        Ok(true)
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Failed to update remote config: {error_text}"))
    }
}

#[tauri::command]
pub async fn rclone_delete_remote(
    name: String,
    _state: State<'_, AppState>,
) -> Result<bool, String> {
    let client = Client::new();

    let response = client
        .post(format!("{RCLONE_API_BASE}/config/delete"))
        .header("Authorization", RCLONE_AUTH)
        .header("Content-Type", "application/json")
        .json(&json!({ "name": name }))
        .send()
        .await
        .map_err(|e| format!("Failed to delete remote config: {e}"))?;

    if response.status().is_success() {
        Ok(true)
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Failed to delete remote config: {error_text}"))
    }
}

#[tauri::command]
pub async fn rclone_mount_remote(
    mount_request: RcloneMountRequest,
    _state: State<'_, AppState>,
) -> Result<bool, String> {
    let client = Client::new();

    let response = client
        .post(format!("{RCLONE_API_BASE}/mount/mount"))
        .header("Authorization", RCLONE_AUTH)
        .header("Content-Type", "application/json")
        .json(&mount_request)
        .send()
        .await
        .map_err(|e| format!("Failed to mount remote: {e}"))?;

    if response.status().is_success() {
        Ok(true)
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Failed to mount remote: {error_text}"))
    }
}

#[tauri::command]
pub async fn rclone_unmount_remote(
    mount_point: String,
    _state: State<'_, AppState>,
) -> Result<bool, String> {
    let client = Client::new();

    let response = client
        .post(format!("{RCLONE_API_BASE}/mount/unmount"))
        .header("Authorization", RCLONE_AUTH)
        .header("Content-Type", "application/json")
        .json(&json!({ "mountPoint": mount_point }))
        .send()
        .await
        .map_err(|e| format!("Failed to unmount remote: {e}"))?;

    if response.status().is_success() {
        Ok(true)
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Failed to unmount remote: {error_text}"))
    }
}

#[tauri::command]
pub async fn create_rclone_mount_remote_process(
    config: ProcessConfig,
    _state: State<'_, AppState>,
) -> Result<ProcessConfig, String> {
    let binary_path =
        get_rclone_binary_path().map_err(|e| format!("Failed to get rclone binary path: {e}"))?;
    let log_file_path =
        get_app_logs_dir().map_err(|e| format!("Failed to get app logs directory: {e}"))?;
    let log_file_path = log_file_path.join("process_rclone.log");
    let rclone_conf_path = binary_path
        .parent()
        .map(|p| p.join("rclone.conf"))
        .ok_or_else(|| "Failed to determine rclone.conf path".to_string())?;

    let api_key = get_api_key();
    let port = get_server_port();
    let mut args: Vec<String> = vec![
        "mount".into(),
        "--config".into(),
        rclone_conf_path.to_string_lossy().into_owned(),
    ];
    args.extend(config.args.clone());

    let config = ProcessConfig {
        id: config.id.clone(),
        name: config.name.clone(),
        bin_path: binary_path.to_string_lossy().into_owned(),
        args,
        log_file: log_file_path.to_string_lossy().into_owned(),
        working_dir: binary_path
            .parent()
            .map(|p| p.to_string_lossy().into_owned()),
        env_vars: config.env_vars.clone(),
        auto_restart: true,
        auto_start: config.auto_start,
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
            "Failed to create Rclone Mount Remote process: {}",
            response.status()
        ))
    }
}

#[tauri::command]
pub async fn check_mount_status(
    mount_point: String,
    _state: State<'_, AppState>,
) -> Result<bool, String> {
    let path = Path::new(&mount_point);
    if !path.exists() {
        return Ok(false);
    }
    #[cfg(target_os = "windows")]
    {
        if mount_point.len() == 2 && mount_point.ends_with(':') {
            let drive_path = format!("{mount_point}\\");
            match fs::read_dir(&drive_path) {
                Ok(_) => return Ok(true),
                Err(_) => return Ok(false),
            }
        }
        match fs::read_dir(&mount_point) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        match fs::read_dir(&mount_point) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[tauri::command]
pub async fn get_mount_info_list(
    state: State<'_, AppState>,
) -> Result<Vec<RcloneMountInfo>, String> {
    let process_list = get_process_list(state.clone()).await?;
    let mut mount_infos = Vec::new();

    for process in process_list {
        if process.name.starts_with("rclone_mount_") {
            let args = &process.config.args;

            if args.len() >= 3 && args[0] == "mount" {
                let remote_path = args[3].clone();
                let mount_point = args[4].clone();

                let mount_status =
                    match check_mount_status(mount_point.clone(), state.clone()).await {
                        Ok(is_mounted) => {
                            if process.is_running {
                                if is_mounted { "mounted" } else { "mounting" }
                            } else if is_mounted {
                                "unmounting"
                            } else {
                                "unmounted"
                            }
                        }
                        Err(_) => "error",
                    };

                mount_infos.push(RcloneMountInfo {
                    name: remote_path.split(':').next().unwrap_or("").to_string(),
                    process_id: process.id,
                    remote_path,
                    mount_point,
                    status: mount_status.to_string(),
                });
            }
        }
    }
    Ok(mount_infos)
}
