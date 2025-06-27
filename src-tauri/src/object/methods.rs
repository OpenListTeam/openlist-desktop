use parking_lot::RwLock;
use std::sync::Arc;
use tauri::AppHandle;

use crate::conf::config::MergedSettings;
use crate::object::structs::AppState;

impl AppState {
    pub fn new() -> Self {
        Self {
            app_settings: Arc::new(RwLock::new(None)),
            app_handle: Arc::new(RwLock::new(None)),
        }
    }

    pub fn init(&self, app_handle: &AppHandle) -> Result<(), String> {
        {
            let mut handle = self.app_handle.write();
            *handle = Some(app_handle.clone());
        }
        self.load_settings()?;
        Ok(())
    }

    pub fn load_settings(&self) -> Result<(), String> {
        match MergedSettings::load() {
            Ok(settings) => {
                let mut app_settings = self.app_settings.write();
                *app_settings = Some(settings);
                log::info!("Settings loaded successfully");
                Ok(())
            }
            Err(e) => {
                log::warn!("Failed to load settings, using defaults: {e}");
                let default_settings = MergedSettings::default();
                let mut app_settings = self.app_settings.write();
                *app_settings = Some(default_settings);
                Ok(())
            }
        }
    }

    pub fn get_settings(&self) -> Option<MergedSettings> {
        let app_settings = self.app_settings.read();
        app_settings.clone()
    }

    pub fn update_settings(&self, settings: MergedSettings) {
        let mut app_settings = self.app_settings.write();
        *app_settings = Some(settings);
    }
}
