use crate::conf::rclone::RcloneConfig;
use crate::{conf::core::OpenListCoreConfig, utils::path::app_config_file_path};
use std::path::PathBuf;

use super::app::AppConfig;
use serde::{Deserialize, Serialize};

#[allow(unused)]
pub static OPENLIST_CORE_CONFIG: &str = "data/config.json";
#[allow(unused)]
pub static OPENLIST_DESKTOP_SETTINGS_FILE_NAME: &str = "settings.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MergedSettings {
    pub openlist: OpenListCoreConfig,
    pub rclone: RcloneConfig,
    pub app: AppConfig,
}

impl MergedSettings {
    pub fn new() -> Self {
        Self {
            openlist: OpenListCoreConfig::new(),
            rclone: RcloneConfig::new(),
            app: AppConfig::new(),
        }
    }

    fn get_data_config_path() -> Result<PathBuf, String> {
        let app_dir = std::env::current_exe()
            .map_err(|e| format!("Failed to get current exe path: {}", e))?
            .parent()
            .ok_or("Failed to get parent directory")?
            .to_path_buf();
        Ok(app_dir.join("data").join("config.json"))
    }

    fn read_data_config() -> Result<serde_json::Value, String> {
        let path = Self::get_data_config_path()?;
        if !path.exists() {
            return Err("data/config.json does not exist".to_string());
        }

        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    fn get_port_from_data_config() -> Result<Option<u16>, String> {
        let config = Self::read_data_config()?;
        Ok(config
            .get("scheme")
            .and_then(|scheme| scheme.get("http_port"))
            .and_then(|port| port.as_u64())
            .map(|port| port as u16))
    }

    pub fn save(&self) -> Result<(), String> {
        let path = app_config_file_path().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load() -> Result<Self, String> {
        let path = app_config_file_path().map_err(|e| e.to_string())?;
        let mut merged_settings = if !path.exists() {
            std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
            let new_settings = Self::new();
            std::fs::write(
                &path,
                serde_json::to_string_pretty(&new_settings).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            new_settings
        } else {
            let config = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
            serde_json::from_str(&config).map_err(|e| e.to_string())?
        };

        if let Ok(data_port) = Self::get_port_from_data_config() {
            if let Some(port) = data_port {
                if merged_settings.openlist.port != port {
                    merged_settings.openlist.port = port;
                    merged_settings.save()?;
                }
            }
        }

        Ok(merged_settings)
    }

    pub fn default() -> Self {
        Self::new()
    }
}
