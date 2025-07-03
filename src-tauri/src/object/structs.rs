use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::conf::config::MergedSettings;

#[derive(Debug, Serialize, Clone)]
pub struct ServiceStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub port: Option<u16>,
}

#[derive(Debug, Serialize, Clone)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RcloneMountInfo {
    pub name: String,
    pub process_id: String,
    pub remote_path: String,
    pub mount_point: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransferStats {
    pub read: u64,
    pub write: u64,
    pub errors: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RcloneRemoteListResponse {
    pub remotes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RcloneMountListResponse {
    pub mounts: Vec<RcloneMountStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RcloneMountStatus {
    pub mount_point: String,
    pub fs: String,
    pub mounted: bool,
}

pub struct AppState {
    pub app_settings: Arc<RwLock<Option<MergedSettings>>>,
    pub app_handle: Arc<RwLock<Option<AppHandle>>>,
}
