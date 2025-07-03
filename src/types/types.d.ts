declare const OS_PLATFORM: Platform

interface IRemoteConfig {
  [key: string]: RcloneWebdavConfig
}

interface OpenListCoreConfig {
  port: number
  api_token: string
  auto_launch: boolean
  ssl_enabled: boolean
}

interface RcloneConfig {
  config?: any // Flexible JSON object for rclone configuration
}

interface RcloneWebdavConfig {
  url: string
  vendor?: string
  user: string
  pass: string
}

interface RcloneFormConfig {
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

interface RcloneMountInfo {
  name: string
  processId: string
  remotePath: string
  mountPoint: string
  status: 'mounted' | 'unmounted' | 'mounting' | 'unmounting' | 'error'
}

interface AppConfig {
  theme?: 'light' | 'dark' | 'auto'
  monitor_interval?: number
  auto_update_enabled?: boolean
}

interface MergedSettings {
  openlist: OpenListCoreConfig
  rclone: RcloneConfig
  app: AppConfig
}

interface OpenListCoreStatus {
  running: boolean
  pid?: number
  port?: number
}

interface MountStatus {
  mounted: boolean
  mount_path?: string
  remote_name?: string
}
interface FileItem {
  name: string
  path: string
  isDir: boolean
  size?: number
  modified?: string
  type?: string
}

interface ProcessConfig {
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

interface ProcessStatus {
  id: string
  name: string
  is_running: boolean
  pid?: number
  started_at?: number
  restart_count: number
  last_exit_code?: number
  config: ProcessConfig
}

interface UpdateAsset {
  name: string
  url: string
  size: number
  platform: string
  type: 'exe' | 'deb' | 'rpm' | 'dmg' | string
}

interface UpdateCheck {
  hasUpdate: boolean
  currentVersion: string
  latestVersion: string
  releaseDate: string
  releaseNotes: string
  assets: UpdateAsset[]
}

interface DownloadProgress {
  downloaded: number
  total: number
  percentage: number
  speed: number
}
