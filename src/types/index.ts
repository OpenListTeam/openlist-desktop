export interface OpenListCoreConfig {
  port: number
  api_token: string
  auto_launch: boolean
  ssl_enabled: boolean
}

export interface RcloneConfig {
  config?: any // Flexible JSON object for rclone configuration
}

export interface RcloneWebdavConfig {
  url: string
  vendor?: string
  user: string
  pass: string
}

export interface RcloneFormConfig {
  name: string
  type: string
  url: string
  vendor?: string
  user: string
  pass: string
  mountPoint?: string
  volumeName?: string
  extraFlags?: string[]
  autoMount: boolean
}

export interface RcloneMountInfo {
  name: string
  processId: string
  remotePath: string
  mountPoint: string
  status: 'mounted' | 'unmounted' | 'mounting' | 'unmounting' | 'error'
}

export interface RcloneCreateRemoteRequest {
  name: string
  type: string
  parameters: {
    url: string
    vendor?: string
    user: string
    pass: string
  }
}

export interface RcloneMountRequest {
  fs: string
  mountPoint: string
  mountType?: string
  vfsOpt?: Record<string, any>
  mountOpt?: {
    ExtraFlags?: string[]
    ExtraOptions?: string[]
    VolumeName?: string
  }
}

export interface AppConfig {
  theme?: 'light' | 'dark' | 'auto'
  monitor_interval?: number
  auto_update_enabled?: boolean
}

export interface MergedSettings {
  openlist: OpenListCoreConfig
  rclone: RcloneConfig
  app: AppConfig
}

export interface OpenListCoreStatus {
  running: boolean
  pid?: number
  port?: number
}

export interface MountStatus {
  mounted: boolean
  mount_path?: string
  remote_name?: string
}
export interface FileItem {
  name: string
  path: string
  isDir: boolean
  size?: number
  modified?: string
  type?: string
}

export interface AppState {
  serviceStatus: OpenListCoreStatus
  mountStatus: MountStatus
  logs: string[]
  settings: MergedSettings
  currentPath: string
  files: FileItem[]
  loading: boolean
  error?: string
}

export interface TauriResponse<T = any> {
  success: boolean
  data?: T
  error?: string
}

// Events
export interface LogEvent {
  timestamp: string
  level: string
  source: string
  message: string
}

export interface ServiceEvent {
  type: 'started' | 'stopped' | 'error'
  data?: any
}

export interface MountEvent {
  type: 'mounted' | 'unmounted' | 'error'
  data?: any
}

export interface ProcessConfig {
  id: string
  name: string
  bin_path: string
  args: string[]
  log_file: string
  working_dir?: string
  env_vars?: Record<string, string>
  auto_restart: boolean
  auto_start: boolean
  run_as_admin: boolean
  created_at: number
  updated_at: number
}

export interface ProcessStatus {
  id: string
  name: string
  is_running: boolean
  pid?: number
  started_at?: number
  restart_count: number
  last_exit_code?: number
  config: ProcessConfig
}

export interface GitHubRelease {
  tag_name: string
  name: string
  body: string
  published_at: string
  assets: GitHubAsset[]
  prerelease: boolean
  draft: boolean
}

export interface GitHubAsset {
  id: number
  name: string
  size: number
  download_count: number
  browser_download_url: string
  content_type: string
}

export interface UpdateAsset {
  name: string
  url: string
  size: number
  platform: string
  type: 'exe' | 'deb' | 'rpm' | 'dmg' | string
}

export interface UpdateCheck {
  hasUpdate: boolean
  currentVersion: string
  latestVersion: string
  releaseDate: string
  releaseNotes: string
  assets: UpdateAsset[]
}

export interface DownloadProgress {
  downloaded: number
  total: number
  percentage: number
  speed: number
}

export interface UpdateInfo {
  version: string
  date?: string
  body: string
  available: boolean
}
