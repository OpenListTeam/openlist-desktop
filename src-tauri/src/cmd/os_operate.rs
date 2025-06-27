use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, State};

use crate::cmd::http_api::{get_process_list, start_process, stop_process};
use crate::object::structs::{AppState, FileItem};
use crate::utils::path::{get_openlist_binary_path, get_rclone_binary_path};

fn normalize_path(path: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        let normalized = path.replace('/', "\\");
        if normalized.len() == 2 && normalized.chars().nth(1) == Some(':') {
            format!("{normalized}\\")
        } else if normalized.len() > 2
            && normalized.chars().nth(1) == Some(':')
            && normalized.chars().nth(2) != Some('\\')
        {
            let drive = &normalized[..2];
            let rest = &normalized[2..];
            format!("{drive}\\{rest}")
        } else {
            normalized
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        path.to_string()
    }
}

#[tauri::command]
pub async fn open_folder(path: String) -> Result<bool, String> {
    let normalized_path = normalize_path(&path);
    let path_buf = PathBuf::from(normalized_path);
    if !path_buf.exists() {
        return Err(format!("Path does not exist: {}", path_buf.display()));
    }
    open::that(path_buf.as_os_str()).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub async fn open_file(path: String) -> Result<bool, String> {
    let normalized_path = normalize_path(&path);
    let path_buf = PathBuf::from(normalized_path);
    if !path_buf.exists() {
        return Err(format!("File does not exist: {}", path_buf.display()));
    }

    open::that_detached(path_buf.as_os_str()).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub async fn open_url(url: String) -> Result<bool, String> {
    open::that(url).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn select_directory(title: String, app_handle: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let dir_path = app_handle
        .dialog()
        .file()
        .set_title(&title)
        .blocking_pick_folder();

    Ok(dir_path.map(|path| path.to_string()))
}

#[tauri::command]
pub async fn list_files(
    path: String,
    _state: State<'_, AppState>,
) -> Result<Vec<FileItem>, String> {
    let path_buf = PathBuf::from(&path);

    if !path_buf.exists() {
        return Err("Path does not exist".to_string());
    }

    let mut files = Vec::new();

    if path_buf.is_dir() {
        let entries = fs::read_dir(&path_buf).map_err(|e| e.to_string())?;

        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let metadata = entry.metadata().map_err(|e| e.to_string())?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_path = entry.path().to_string_lossy().to_string();

            files.push(FileItem {
                name: file_name,
                path: file_path,
                is_dir: metadata.is_dir(),
                size: if metadata.is_file() {
                    Some(metadata.len())
                } else {
                    None
                },
                modified: metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|duration| {
                        chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                            .unwrap_or_default()
                            .to_rfc3339()
                    }),
            });
        }
    }

    files.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    Ok(files)
}

#[tauri::command]
pub async fn get_available_versions(tool: String) -> Result<Vec<String>, String> {
    let url = match tool.as_str() {
        "openlist" => "https://api.github.com/repos/OpenListTeam/OpenList/releases",
        "rclone" => "https://api.github.com/repos/rclone/rclone/releases",
        _ => return Err("Unsupported tool".to_string()),
    };

    let client = reqwest::Client::builder()
        .user_agent("OpenList Desktop/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    let releases: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

    let versions = releases
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .take(10)
        .filter_map(|release| release["tag_name"].as_str())
        .map(|tag| tag.to_string())
        .collect();

    Ok(versions)
}

#[tauri::command]
pub async fn update_tool_version(
    tool: String,
    version: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("Updating {tool} to version {version}");

    let process_list = get_process_list(state.clone())
        .await
        .map_err(|e| format!("Failed to get process list: {e}"))?;

    let process_name = match tool.as_str() {
        "openlist" => "single_openlist_core_process",
        "rclone" => "single_rclone_backend_process",
        _ => return Err("Unsupported tool".to_string()),
    };

    let running_process = process_list.iter().find(|p| p.config.name == process_name);

    let was_running = running_process.map(|p| p.is_running).unwrap_or(false);
    let process_id = running_process.map(|p| p.config.id.clone());

    if was_running {
        if let Some(pid) = &process_id {
            log::info!("Stopping {tool} process with ID: {pid}");
            match tool.as_str() {
                "openlist" | "rclone" => {
                    stop_process(pid.clone(), state.clone())
                        .await
                        .map_err(|e| format!("Failed to stop process: {e}"))?;
                }
                _ => return Err("Unsupported tool".to_string()),
            }
            log::info!("Successfully stopped {tool} process");
        }
    }

    let result = download_and_replace_binary(&tool, &version).await;

    match result {
        Ok(_) => {
            log::info!("Successfully downloaded and replaced {tool} binary");

            if was_running {
                if let Some(pid) = &process_id {
                    log::info!("Starting {tool} process with ID: {pid}");
                    match tool.as_str() {
                        "openlist" | "rclone" => {
                            start_process(pid.clone(), state.clone())
                                .await
                                .map_err(|e| format!("Failed to start {tool} process: {e}"))?;
                        }
                        _ => return Err("Unsupported tool".to_string()),
                    }
                    log::info!("Successfully restarted {tool} process");
                }
            }

            Ok(format!("Successfully updated {tool} to {version}"))
        }
        Err(e) => {
            log::error!("Failed to update {tool} binary: {e}");

            if was_running {
                if let Some(pid) = &process_id {
                    log::info!(
                        "Attempting to restart {tool} with previous binary after update failure"
                    );
                    match tool.as_str() {
                        "openlist" | "rclone" => {
                            let _ = start_process(pid.clone(), state.clone()).await;
                        }
                        _ => {}
                    }
                }
            }

            Err(format!("Failed to update {tool} to {version}: {e}"))
        }
    }
}

async fn download_and_replace_binary(tool: &str, version: &str) -> Result<(), String> {
    let platform = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let platform_arch = format!(
        "{}-{}",
        match platform {
            "windows" => "win32",
            "macos" => "darwin",
            "linux" => "linux",
            _ => return Err(format!("Unsupported platform: {platform}")),
        },
        match arch {
            "x86_64" => "x64",
            "x86" => "ia32",
            "aarch64" => "arm64",
            "arm" => "arm",
            _ => return Err(format!("Unsupported architecture: {arch}")),
        }
    );

    log::info!("Detected platform: {platform_arch}");

    let (binary_path, download_info) = match tool {
        "openlist" => {
            let path = get_openlist_binary_path()
                .map_err(|e| format!("Failed to get OpenList binary path: {e}"))?;
            let info = get_openlist_download_info(&platform_arch, version)?;
            (path, info)
        }
        "rclone" => {
            let path = get_rclone_binary_path()
                .map_err(|e| format!("Failed to get Rclone binary path: {e}"))?;
            let info = get_rclone_download_info(&platform_arch, version)?;
            (path, info)
        }
        _ => return Err("Unsupported tool".to_string()),
    };

    log::info!("Downloading {} from: {}", tool, download_info.download_url);

    let temp_dir = std::env::temp_dir().join(format!("{tool}-update-{version}"));
    fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp directory: {e}"))?;

    let archive_path = temp_dir.join(&download_info.archive_name);
    download_file(&download_info.download_url, &archive_path).await?;

    let extracted_binary_path = extract_binary(
        &archive_path,
        &temp_dir,
        &download_info.executable_name,
        tool,
    )
    .await?;

    let backup_path = binary_path.with_extension(format!(
        "{}.backup",
        binary_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    ));

    if binary_path.exists() {
        fs::copy(&binary_path, &backup_path)
            .map_err(|e| format!("Failed to backup current binary: {e}"))?;
    }

    fs::copy(&extracted_binary_path, &binary_path).map_err(|e| {
        if backup_path.exists() {
            let _ = fs::copy(&backup_path, &binary_path);
            let _ = fs::remove_file(&backup_path);
        }
        format!("Failed to replace binary: {e}")
    })?;

    if backup_path.exists() {
        let _ = fs::remove_file(&backup_path);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)
            .map_err(|e| format!("Failed to get binary metadata: {e}"))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)
            .map_err(|e| format!("Failed to set executable permissions: {e}"))?;
    }

    let _ = fs::remove_file(&extracted_binary_path);
    let _ = fs::remove_dir_all(&temp_dir);

    log::info!("Successfully replaced {tool} binary");
    Ok(())
}

struct DownloadInfo {
    download_url: String,
    archive_name: String,
    executable_name: String,
}

fn get_openlist_download_info(platform_arch: &str, version: &str) -> Result<DownloadInfo, String> {
    let arch_map = get_openlist_arch_mapping(platform_arch)?;
    let is_windows = platform_arch.starts_with("win32");
    let is_unix = platform_arch.starts_with("darwin") || platform_arch.starts_with("linux");

    let archive_ext = if is_unix { "tar.gz" } else { "zip" };
    let exe_ext = if is_windows { ".exe" } else { "" };

    let archive_name = format!("openlist-{arch_map}.{archive_ext}");
    let executable_name = format!("openlist{exe_ext}");
    let download_url = format!(
        "https://github.com/OpenListTeam/OpenList/releases/download/{version}/{archive_name}"
    );

    Ok(DownloadInfo {
        download_url,
        archive_name,
        executable_name,
    })
}

fn get_rclone_download_info(platform_arch: &str, version: &str) -> Result<DownloadInfo, String> {
    let arch_map = get_rclone_arch_mapping(platform_arch)?;
    let is_windows = platform_arch.starts_with("win32");

    let exe_ext = if is_windows { ".exe" } else { "" };
    let archive_name = format!("rclone-{version}-{arch_map}.zip");
    let executable_name = format!("rclone{exe_ext}");
    let download_url =
        format!("https://github.com/rclone/rclone/releases/download/{version}/{archive_name}");

    Ok(DownloadInfo {
        download_url,
        archive_name,
        executable_name,
    })
}

fn get_openlist_arch_mapping(platform_arch: &str) -> Result<&'static str, String> {
    match platform_arch {
        "win32-x64" => Ok("windows-amd64"),
        "win32-ia32" => Ok("windows-386"),
        "win32-arm64" => Ok("windows-arm64"),
        "darwin-x64" => Ok("darwin-amd64"),
        "darwin-arm64" => Ok("darwin-arm64"),
        "linux-x64" => Ok("linux-amd64"),
        "linux-ia32" => Ok("linux-386"),
        "linux-arm64" => Ok("linux-arm64"),
        "linux-arm" => Ok("linux-arm-7"),
        _ => Err(format!(
            "Unsupported platform architecture: {platform_arch}"
        )),
    }
}

fn get_rclone_arch_mapping(platform_arch: &str) -> Result<&'static str, String> {
    match platform_arch {
        "win32-x64" => Ok("windows-amd64"),
        "win32-ia32" => Ok("windows-386"),
        "win32-arm64" => Ok("windows-arm64"),
        "darwin-x64" => Ok("osx-amd64"),
        "darwin-arm64" => Ok("osx-arm64"),
        "linux-x64" => Ok("linux-amd64"),
        "linux-ia32" => Ok("linux-386"),
        "linux-arm64" => Ok("linux-arm64"),
        "linux-arm" => Ok("linux-arm-v7"),
        _ => Err(format!(
            "Unsupported platform architecture: {platform_arch}"
        )),
    }
}

async fn download_file(url: &str, path: &PathBuf) -> Result<(), String> {
    log::info!("Downloading file from: {url}");

    let client = reqwest::Client::builder()
        .user_agent("OpenList Desktop/1.0")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download file: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response bytes: {e}"))?;

    fs::write(path, &bytes).map_err(|e| format!("Failed to write file: {e}"))?;

    log::info!("Downloaded file to: {path:?}");
    Ok(())
}

async fn extract_binary(
    archive_path: &PathBuf,
    extract_dir: &Path,
    executable_name: &str,
    tool: &str,
) -> Result<PathBuf, String> {
    log::info!("Extracting archive: {archive_path:?}");

    let archive_name = archive_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid archive path")?;

    if archive_name.ends_with(".zip") {
        extract_zip(archive_path, extract_dir, executable_name, tool)
    } else if archive_name.ends_with(".tar.gz") {
        extract_tar_gz(archive_path, extract_dir, executable_name, tool)
    } else {
        Err(format!("Unsupported archive format: {archive_name}"))
    }
}

fn extract_zip(
    archive_path: &PathBuf,
    extract_dir: &Path,
    executable_name: &str,
    tool: &str,
) -> Result<PathBuf, String> {
    let file = fs::File::open(archive_path).map_err(|e| format!("Failed to open zip file: {e}"))?;

    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip archive: {e}"))?;

    let mut executable_path = None;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {e}"))?;

        let file_name = file.name();

        let is_target_executable = if tool == "rclone" {
            file_name.ends_with(executable_name) && file_name.contains("rclone")
        } else {
            file_name == executable_name || file_name.ends_with(&format!("/{executable_name}"))
        };

        if is_target_executable {
            let output_path = extract_dir.join(executable_name);
            let mut output_file = fs::File::create(&output_path)
                .map_err(|e| format!("Failed to create output file: {e}"))?;

            std::io::copy(&mut file, &mut output_file)
                .map_err(|e| format!("Failed to extract file: {e}"))?;

            executable_path = Some(output_path);
            break;
        }
    }

    executable_path
        .ok_or_else(|| format!("Executable '{executable_name}' not found in zip archive"))
}

fn extract_tar_gz(
    archive_path: &PathBuf,
    extract_dir: &Path,
    executable_name: &str,
    _tool: &str,
) -> Result<PathBuf, String> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let file =
        fs::File::open(archive_path).map_err(|e| format!("Failed to open tar.gz file: {e}"))?;

    let gz_decoder = GzDecoder::new(file);
    let mut archive = Archive::new(gz_decoder);

    let mut executable_path = None;

    for entry in archive
        .entries()
        .map_err(|e| format!("Failed to read tar entries: {e}"))?
    {
        let mut entry = entry.map_err(|e| format!("Failed to read tar entry: {e}"))?;
        let path = entry
            .path()
            .map_err(|e| format!("Failed to get entry path: {e}"))?;

        if let Some(file_name) = path.file_name() {
            if file_name == executable_name {
                let output_path = extract_dir.join(executable_name);
                let mut output_file = fs::File::create(&output_path)
                    .map_err(|e| format!("Failed to create output file: {e}"))?;

                std::io::copy(&mut entry, &mut output_file)
                    .map_err(|e| format!("Failed to extract file: {e}"))?;

                executable_path = Some(output_path);
                break;
            }
        }
    }

    executable_path
        .ok_or_else(|| format!("Executable '{executable_name}' not found in tar.gz archive"))
}
