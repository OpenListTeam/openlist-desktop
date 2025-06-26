use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RcloneConfig {
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RcloneWebdavConfig {
    pub url: String,
    pub vendor: Option<String>,
    pub user: String,
    pub pass: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RcloneCreateRemoteRequest {
    pub name: String,
    pub r#type: String,
    pub parameters: RcloneWebdavConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RcloneRemoteParameters {
    pub url: String,
    pub vendor: Option<String>,
    pub user: String,
    pub pass: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RcloneMountRequest {
    pub fs: String,
    pub mount_point: String,
    pub mount_type: Option<String>,
    pub vfs_opt: Option<HashMap<String, String>>,
    pub mount_opt: Option<RcloneMountOptions>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RcloneMountOptions {
    #[serde(rename = "ExtraFlags")]
    pub extra_flags: Option<Vec<String>>,
    #[serde(rename = "ExtraOptions")]
    pub extra_options: Option<Vec<String>>,
    #[serde(rename = "VolumeName")]
    pub volume_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RcloneApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl RcloneConfig {
    pub fn new() -> Self {
        Self {
            config: serde_json::Value::Object(Default::default()),
        }
    }
}
