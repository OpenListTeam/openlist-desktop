use crate::{cmd::config::save_settings, object::structs::AppState};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRelease {
    tag_name: String,
    name: String,
    body: String,
    published_at: String,
    assets: Vec<GitHubAsset>,
    prerelease: bool,
    draft: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubAsset {
    id: u64,
    name: String,
    size: u64,
    download_count: u64,
    browser_download_url: String,
    content_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAsset {
    name: String,
    url: String,
    size: u64,
    platform: String,
    #[serde(rename = "type")]
    asset_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCheck {
    #[serde(rename = "hasUpdate")]
    has_update: bool,
    #[serde(rename = "currentVersion")]
    current_version: String,
    #[serde(rename = "latestVersion")]
    latest_version: String,
    #[serde(rename = "releaseDate")]
    release_date: String,
    #[serde(rename = "releaseNotes")]
    release_notes: String,
    assets: Vec<UpdateAsset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadProgress {
    downloaded: u64,
    total: u64,
    percentage: f64,
    speed: f64,
}

fn get_current_platform() -> String {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    match os {
        "windows" => format!("{arch}-pc-windows-msvc"),
        "macos" => format!("{arch}-apple-darwin"),
        "linux" => format!("{arch}-unknown-linux-gnu"),
        _ => format!("{arch}-{os}"),
    }
}

fn filter_assets_for_platform(assets: &[GitHubAsset]) -> Vec<UpdateAsset> {
    let platform = get_current_platform();
    let os = env::consts::OS;

    assets
        .iter()
        .filter_map(|asset| {
            let name = asset.name.to_lowercase();

            let is_for_platform = match os {
                "windows" => {
                    name.contains("windows")
                        || name.contains("win")
                        || name.contains("msvc")
                        || name.contains("exe")
                }
                "macos" => {
                    name.contains("darwin")
                        || name.contains("macos")
                        || name.contains("mac")
                        || name.contains("dmg")
                }
                "linux" => {
                    name.contains("linux")
                        || name.contains("gnu")
                        || name.contains("deb")
                        || name.contains("rpm")
                }
                _ => false,
            };

            if !is_for_platform {
                return None;
            }

            let asset_type = if name.ends_with(".exe") {
                "exe"
            } else if name.ends_with(".deb") {
                "deb"
            } else if name.ends_with(".rpm") {
                "rpm"
            } else if name.ends_with(".dmg") {
                "dmg"
            } else {
                return None;
            };

            Some(UpdateAsset {
                name: asset.name.clone(),
                url: asset.browser_download_url.clone(),
                size: asset.size,
                platform: platform.clone(),
                asset_type: asset_type.to_string(),
            })
        })
        .collect()
}

fn compare_versions(current: &str, latest: &str) -> bool {
    let current = current.trim_start_matches('v');
    let latest = latest.trim_start_matches('v');

    let parse_version = |version: &str| -> Vec<u32> {
        version
            .split('.')
            .filter_map(|part| {
                let numeric_part = part.split('-').next().unwrap_or(part);
                numeric_part.parse::<u32>().ok()
            })
            .collect()
    };

    let current_parts = parse_version(current);
    let latest_parts = parse_version(latest);

    let max_len = current_parts.len().max(latest_parts.len());
    for i in 0..max_len {
        let current_part = current_parts.get(i).unwrap_or(&0);
        let latest_part = latest_parts.get(i).unwrap_or(&0);

        match latest_part.cmp(current_part) {
            std::cmp::Ordering::Greater => return true,
            std::cmp::Ordering::Less => return false,
            std::cmp::Ordering::Equal => continue,
        }
    }

    false
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateCheck, String> {
    log::info!("Checking for updates...");

    let client = Client::new();
    let url = "https://api.github.com/repos/OpenListTeam/openlist-desktop/releases/latest";

    let response = client
        .get(url)
        .header("User-Agent", "OpenList-Desktop")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("Network error while checking for updates: {e}");
            log::error!("{error_msg}");
            error_msg
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_msg = if status.as_u16() == 404 {
            "Repository not found. Please check the repository URL.".to_string()
        } else if status.as_u16() == 403 {
            "API rate limit exceeded. Please try again later.".to_string()
        } else {
            format!(
                "GitHub API returned status: {} ({})",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown")
            )
        };
        log::error!("{error_msg}");
        return Err(error_msg);
    }

    let release: GitHubRelease = response.json().await.map_err(|e| {
        log::error!("Failed to parse GitHub response: {e}");
        format!("Failed to parse update information: {e}")
    })?;

    let current_version = env!("CARGO_PKG_VERSION");
    let latest_version = release.tag_name.as_str();

    let has_update = compare_versions(current_version, latest_version);
    let assets = filter_assets_for_platform(&release.assets);

    log::info!(
        "Update check result: current={}, latest={}, has_update={}, assets_count={}",
        current_version,
        latest_version,
        has_update,
        assets.len()
    );

    Ok(UpdateCheck {
        has_update,
        current_version: current_version.to_string(),
        latest_version: latest_version.to_string(),
        release_date: release.published_at,
        release_notes: release.body,
        assets,
    })
}

#[tauri::command]
pub async fn download_update(
    app: AppHandle,
    asset_url: String,
    asset_name: String,
) -> Result<String, String> {
    log::info!("Starting download of update: {asset_name}");

    let client = Client::new();

    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(&asset_name);

    log::info!("Downloading to: {file_path:?}");

    let mut response = client
        .get(&asset_url)
        .header("User-Agent", "OpenList-Desktop")
        .timeout(Duration::from_secs(9000))
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to start download: {e}");
            log::error!("{error_msg}");
            error_msg
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_msg = if status.as_u16() == 404 {
            "Download file not found. The release asset may have been removed.".to_string()
        } else {
            format!(
                "Download failed with status: {} ({})",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown")
            )
        };
        log::error!("{error_msg}");
        return Err(error_msg);
    }

    let total_size = response.content_length().unwrap_or(0);
    log::info!("Download size: {total_size} bytes");

    let mut file = tokio::fs::File::create(&file_path).await.map_err(|e| {
        log::error!("Failed to create download file: {e}");
        format!("Failed to create file: {e}")
    })?;

    let mut downloaded = 0u64;
    let mut last_progress_time = std::time::Instant::now();
    let mut last_downloaded = 0u64;

    while let Some(chunk) = response.chunk().await.map_err(|e| {
        log::error!("Download chunk error: {e}");
        format!("Download error: {e}")
    })? {
        file.write_all(&chunk).await.map_err(|e| {
            log::error!("File write error: {e}");
            format!("File write error: {e}")
        })?;

        downloaded += chunk.len() as u64;

        let now = std::time::Instant::now();
        if now.duration_since(last_progress_time).as_secs() >= 1 || downloaded >= total_size {
            let elapsed = now.duration_since(last_progress_time).as_secs_f64();
            let speed = if elapsed > 0.0 {
                (downloaded - last_downloaded) as f64 / elapsed
            } else {
                0.0
            };

            let percentage = if total_size > 0 {
                (downloaded as f64 / total_size as f64) * 100.0
            } else {
                0.0
            };

            let progress = DownloadProgress {
                downloaded,
                total: total_size,
                percentage,
                speed,
            };

            if let Err(e) = app.emit("download-progress", &progress) {
                log::error!("Failed to emit download progress: {e}");
            }

            last_progress_time = now;
            last_downloaded = downloaded;
        }
    }

    file.flush().await.map_err(|e| {
        log::error!("Failed to flush file: {e}");
        format!("File flush error: {e}")
    })?;

    log::info!("Download completed: {downloaded} bytes");

    if let Err(e) = app.emit("update-download-completed", ()) {
        log::error!("Failed to emit download completed event: {e}");
    }

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn install_update_and_restart(
    app: AppHandle,
    installer_path: String,
) -> Result<(), String> {
    log::info!("Installing update from: {installer_path}");

    let path = PathBuf::from(&installer_path);

    if !path.exists() {
        let error_msg = "Installer file not found".to_string();
        log::error!("{error_msg}");
        return Err(error_msg);
    }

    if let Err(e) = app.emit("update-install-started", ()) {
        log::error!("Failed to emit install started event: {e}");
    }

    let result = match env::consts::OS {
        "windows" => install_windows_update(&path).await,
        "macos" => install_macos_update(&path).await,
        "linux" => install_linux_update(&path).await,
        _ => Err("Unsupported platform for auto-update".to_string()),
    };

    match result {
        Ok(_) => {
            log::info!("Update installation started successfully");

            if let Err(e) = app.emit("update-install-completed", ()) {
                log::error!("Failed to emit install completed event: {e}");
            }

            if let Err(e) = app.emit("app-restarting", ()) {
                log::error!("Failed to emit app restarting event: {e}");
            }

            tokio::time::sleep(Duration::from_millis(1000)).await;
            std::process::exit(0);
        }
        Err(e) => {
            log::error!("Update installation failed: {e}");
            if let Err(emit_err) = app.emit("update-install-error", &e) {
                log::error!("Failed to emit install error event: {emit_err}");
            }
            Err(e)
        }
    }
}

async fn install_windows_update(installer_path: &PathBuf) -> Result<(), String> {
    log::info!("Installing Windows update...");

    let mut cmd = Command::new(installer_path);
    cmd.arg("/SILENT");

    let _ = tokio::task::spawn_blocking(move || {
        cmd.spawn()
            .map_err(|e| format!("Failed to start Windows installer: {e}"))
    })
    .await
    .map_err(|e| format!("Task error: {e}"))?;

    Ok(())
}

async fn install_macos_update(installer_path: &PathBuf) -> Result<(), String> {
    log::info!("Installing macOS update...");

    let mut cmd = Command::new("open");
    cmd.arg(installer_path);

    let _ = tokio::task::spawn_blocking(move || {
        cmd.spawn()
            .map_err(|e| format!("Failed to start macOS installer: {e}"))
    })
    .await
    .map_err(|e| format!("Task error: {e}"))?;

    Ok(())
}

async fn install_linux_update(installer_path: &PathBuf) -> Result<(), String> {
    log::info!("Installing Linux update...");

    let extension = installer_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let mut cmd = match extension {
        "deb" => {
            let mut cmd = Command::new("pkexec");
            cmd.args(["dpkg", "-i"]);
            cmd.arg(installer_path);
            cmd
        }
        "rpm" => {
            let mut cmd = Command::new("pkexec");
            cmd.args(["rpm", "-i"]);
            cmd.arg(installer_path);
            cmd
        }
        _ => {
            return Err("Unsupported Linux package format".to_string());
        }
    };

    let _ = tokio::task::spawn_blocking(move || {
        cmd.spawn()
            .map_err(|e| format!("Failed to start Linux installer: {e}"))
    })
    .await
    .map_err(|e| format!("Task error: {e}"))?;

    Ok(())
}

#[tauri::command]
pub async fn get_current_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[tauri::command]
pub async fn set_auto_check_enabled(
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Setting auto-check updates preference to: {enabled}");

    let mut settings = state.get_settings().unwrap_or_else(|| {
        use crate::conf::config::MergedSettings;
        MergedSettings::default()
    });

    settings.app.auto_update_enabled = Some(enabled);
    state.update_settings(settings.clone());
    save_settings(settings, state)
        .await
        .map_err(|e| format!("Failed to save settings: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn is_auto_check_enabled(state: State<'_, AppState>) -> Result<bool, String> {
    let settings = state.get_settings().unwrap_or_else(|| {
        use crate::conf::config::MergedSettings;
        MergedSettings::default()
    });

    Ok(settings.app.auto_update_enabled.unwrap_or(true))
}

pub async fn perform_background_update_check(app: AppHandle) -> Result<(), String> {
    log::debug!("Performing background update check...");

    match check_for_updates().await {
        Ok(update_check) => {
            if update_check.has_update {
                log::info!(
                    "Background check: Update available {} -> {}",
                    update_check.current_version,
                    update_check.latest_version
                );

                if let Err(e) = app.emit("background-update-available", &update_check) {
                    log::error!("Failed to emit background-update-available event: {e}");
                }
            } else {
                log::error!("Background check: App is up to date");
            }
            Ok(())
        }
        Err(e) => {
            log::error!("Background update check failed: {e}");
            Ok(())
        }
    }
}

#[tauri::command]
pub async fn restart_app(app: AppHandle) {
    log::info!("Restarting application...");

    if let Err(e) = app.emit("app-restarting", ()) {
        log::error!("Failed to emit app-restarting event: {e}");
    }

    tokio::time::sleep(Duration::from_millis(500)).await;

    app.restart();
}
