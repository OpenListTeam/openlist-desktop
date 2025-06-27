import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { appDataDir, join } from '@tauri-apps/api/path'

import {
  DownloadProgress,
  FileItem,
  MergedSettings,
  OpenListCoreStatus,
  ProcessConfig,
  ProcessStatus,
  RcloneMountInfo,
  RcloneWebdavConfig,
  TauriResponse,
  UpdateCheck
} from '../types'

export class TauriAPI {
  // openlist desktop service management
  static async checkServiceStatus(): Promise<string> {
    return await invoke('check_service_status')
  }

  static async installOpenListService(): Promise<boolean> {
    return await invoke('install_service')
  }

  static async uninstallOpenListService(): Promise<boolean> {
    return await invoke('uninstall_service')
  }

  static async startOpenListService(): Promise<boolean> {
    return await invoke('start_service')
  }

  static async stopOpenListService(): Promise<boolean> {
    return await invoke('stop_service')
  }

  // http API management
  static async getProcessList(): Promise<ProcessStatus[]> {
    return await invoke('get_process_list')
  }

  static async startProcess(id: string): Promise<boolean> {
    return await invoke('start_process', { id })
  }

  static async stopProcess(id: string): Promise<boolean> {
    return await invoke('stop_process', { id })
  }

  static async restartProcess(id: string): Promise<boolean> {
    return await invoke('restart_process', { id })
  }

  static async updateProcess(id: string, config: Partial<ProcessConfig>): Promise<boolean> {
    return await invoke('update_process', { id, updateConfig: config })
  }

  // OpenList Core management

  static async createOpenListCore(autoStart: boolean): Promise<ProcessConfig> {
    return await invoke('create_openlist_core_process', { autoStart })
  }

  static async getOpenListCoreStatus(): Promise<OpenListCoreStatus> {
    return await invoke('get_openlist_core_status')
  }

  // Rclone management

  static async createRcloneBackend(): Promise<boolean> {
    return await invoke('create_rclone_backend_process')
  }

  static async createAndStartRcloneBackend(): Promise<ProcessConfig> {
    return await invoke('create_and_start_rclone_backend')
  }

  static async isRcloneRunning(): Promise<boolean> {
    return await invoke('get_rclone_backend_status')
  }

  static async createRemoteConfig(name: string, type: string, config: RcloneWebdavConfig): Promise<boolean> {
    return await invoke('rclone_create_remote', { name, type, config })
  }

  static async updateRemoteConfig(name: string, type: string, config: RcloneWebdavConfig): Promise<boolean> {
    return await invoke('rclone_update_remote', { name, type, config })
  }

  static async deleteRemoteConfig(name: string): Promise<boolean> {
    return await invoke('rclone_delete_remote', { name })
  }

  static async listRcloneRemotes(): Promise<string[]> {
    return await invoke('rclone_list_remotes')
  }

  static async listRcloneMounts(): Promise<any> {
    return await invoke('rclone_list_mounts')
  }

  static async checkMountStatus(mountPoint: string): Promise<boolean> {
    return await invoke('check_mount_status', { mountPoint })
  }

  static async getMountInfoList(): Promise<RcloneMountInfo[]> {
    return await invoke('get_mount_info_list')
  }

  static async rcloneListConfig(remoteType: string): Promise<IRemoteConfig> {
    return await invoke('rclone_list_config', { remoteType })
  }

  static async getRcloneBackendStatus(): Promise<boolean> {
    return await invoke('get_rclone_backend_status')
  }

  static async createRcloneMountRemoteProcess(config: ProcessConfig): Promise<ProcessConfig> {
    return await invoke('create_rclone_mount_remote_process', { config })
  }

  // File operations
  static async listFiles(path: string): Promise<FileItem[]> {
    return await invoke('list_files', { path })
  }

  static async openFile(path: string): Promise<boolean> {
    return await invoke('open_file', { path })
  }

  static async openFolder(path: string): Promise<boolean> {
    return await invoke('open_folder', { path })
  }

  static async openUrl(path: string): Promise<boolean> {
    return await invoke('open_url', { path })
  }

  // Settings management
  static async loadSettings(): Promise<MergedSettings | null> {
    return await invoke('load_settings')
  }

  static async saveSettings(settings: MergedSettings): Promise<boolean> {
    return await invoke('save_settings', { settings })
  }

  static async saveSettingsWithUpdatePort(settings: MergedSettings): Promise<boolean> {
    return await invoke('save_settings_with_update_port', { settings })
  }

  static async resetSettings(): Promise<MergedSettings | null> {
    return await invoke('reset_settings')
  }
  // Logs
  static async getLogs(source?: 'openlist' | 'rclone' | 'app' | 'openlist_core'): Promise<string[]> {
    return await invoke('get_logs', { source })
  }

  static async clearLogs(): Promise<boolean> {
    return await invoke('clear_logs')
  }

  static async getAdminPassword(): Promise<string> {
    return await invoke('get_admin_password')
  }

  // Binary management

  static async getBinaryVersion(binary_name: 'openlist' | 'rclone'): Promise<string> {
    return await invoke('get_binary_version', { binaryName: binary_name })
  }

  static async selectDirectory(title: string): Promise<string | null> {
    return await invoke('select_directory', { title })
  }

  static async getAvailableVersions(tool: string): Promise<string[]> {
    return await invoke('get_available_versions', { tool })
  }

  static async updateToolVersion(tool: string, version: string): Promise<string> {
    return await invoke('update_tool_version', { tool, version })
  }

  static async getAppVersion(): Promise<TauriResponse<string>> {
    return await invoke('get_app_version')
  }

  // Utility methods
  static async getDefaultDataDir(): Promise<string> {
    const appData = await appDataDir()
    return await join(appData, 'openlist-desktop')
  }

  static async getDefaultConfigPath(): Promise<string> {
    const appData = await appDataDir()
    return await join(appData, 'openlist-desktop', 'rclone.conf')
  }

  // Auto-startup management
  static async enableAutoStart(): Promise<TauriResponse<void>> {
    return await invoke('enable_auto_start')
  }

  static async disableAutoStart(): Promise<TauriResponse<void>> {
    return await invoke('disable_auto_start')
  }

  static async isAutoStartEnabled(): Promise<TauriResponse<boolean>> {
    return await invoke('is_auto_start_enabled')
  }

  // System tray management
  static async updateTrayMenu(serviceRunning: boolean): Promise<void> {
    return await invoke('update_tray_menu', { serviceRunning })
  }

  static async updateTrayMenuDelayed(serviceRunning: boolean): Promise<void> {
    return await invoke('update_tray_menu_delayed', { serviceRunning })
  }

  static async forceUpdateTrayMenu(serviceRunning: boolean): Promise<void> {
    return await invoke('force_update_tray_menu', { serviceRunning })
  }

  // Tray event listeners
  static async listenToTrayServiceActions(callback: (action: string) => void) {
    return await listen('tray-core-action', event => {
      callback(event.payload as string)
    })
  }

  // Custom Updater management
  static async checkForUpdates(): Promise<UpdateCheck> {
    return await invoke('check_for_updates')
  }

  static async downloadUpdate(assetUrl: string, assetName: string): Promise<string> {
    return await invoke('download_update', { assetUrl, assetName })
  }

  static async installUpdateAndRestart(installerPath: string): Promise<void> {
    return await invoke('install_update_and_restart', { installerPath })
  }

  static async getCurrentVersion(): Promise<string> {
    return await invoke('get_current_version')
  }

  static async setAutoCheckEnabled(enabled: boolean): Promise<void> {
    return await invoke('set_auto_check_enabled', { enabled })
  }

  static async isAutoCheckEnabled(): Promise<boolean> {
    return await invoke('is_auto_check_enabled')
  }

  // Update event listeners
  static async listenToBackgroundUpdateAvailable(callback: (updateCheck: UpdateCheck) => void) {
    return await listen('background-update-available', event => {
      callback(event.payload as UpdateCheck)
    })
  }

  static async listenToDownloadProgress(callback: (progress: DownloadProgress) => void) {
    return await listen('download-progress', event => {
      callback(event.payload as DownloadProgress)
    })
  }

  static async listenToUpdateInstallStarted(callback: () => void) {
    return await listen('update-install-started', () => {
      callback()
    })
  }

  static async listenToUpdateInstallError(callback: (error: string) => void) {
    return await listen('update-install-error', event => {
      callback(event.payload as string)
    })
  }

  static async listenToAppRestarting(callback: () => void) {
    return await listen('app-restarting', () => {
      callback()
    })
  }
}
