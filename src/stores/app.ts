import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { TauriAPI } from '../api/tauri'

type ActionFn<T = any> = () => Promise<T>

export const useAppStore = defineStore('app', () => {
  const settings = ref<MergedSettings>({
    openlist: { port: 5244, api_token: '', auto_launch: false, ssl_enabled: false },
    rclone: { config: {} },
    app: { theme: 'light', monitor_interval: 5000, auto_update_enabled: true }
  })
  const openlistCoreStatus = ref<OpenListCoreStatus>({ running: false })
  const remoteConfigs = ref<IRemoteConfig>({})
  const mountInfos = ref<RcloneMountInfo[]>([])
  const logs = ref<string[]>([])
  const files = ref<FileItem[]>([])
  const currentPath = ref('/')
  const loading = ref(false)
  const error = ref<string | undefined>()

  const defaultRcloneFormConfig: RcloneFormConfig = {
    name: '',
    type: 'webdav',
    url: '',
    vendor: '',
    user: '',
    pass: '',
    mountPoint: '',
    volumeName: '',
    extraFlags: [],
    autoMount: false
  }

  // Computed
  const mountedConfigs = computed(() => mountInfos.value.filter(mount => mount.status === 'mounted'))

  const fullRcloneConfigs = computed<RcloneFormConfig[]>(() => {
    const result: RcloneFormConfig[] = []
    for (const [key, config] of Object.entries(remoteConfigs.value)) {
      let newConfig
      if (settings.value.rclone.config[key]) {
        newConfig = {
          name: key,
          type: 'webdav',
          url: config.url,
          vendor: config.vendor,
          user: config.user,
          pass: config.pass,
          mountPoint: settings.value.rclone.config[key].mountPoint,
          volumeName: settings.value.rclone.config[key].volumeName,
          extraFlags: settings.value.rclone.config[key].extraFlags || [],
          autoMount: settings.value.rclone.config[key].autoMount ?? false
        }
      } else {
        newConfig = {
          ...defaultRcloneFormConfig,
          name: key,
          url: config.url,
          vendor: config.vendor,
          user: config.user,
          pass: config.pass
        } as RcloneFormConfig
      }
      result.push(newConfig)
      settings.value.rclone.config[key] = newConfig
    }
    return result
  })
  const isCoreRunning = computed(() => openlistCoreStatus.value.running)
  const openListCoreUrl = computed(() => {
    const protocol = settings.value.openlist.ssl_enabled ? 'https' : 'http'
    return `${protocol}://localhost:${settings.value.openlist.port}`
  })

  // Helper
  async function withLoading<T>(fn: ActionFn<T>, msg: string): Promise<T> {
    loading.value = true
    try {
      return await fn()
    } catch (e: any) {
      error.value = msg
      console.error(msg, e)
      throw e
    } finally {
      loading.value = false
    }
  }
  // Settings
  const loadSettings = () =>
    withLoading(async () => {
      const res = await TauriAPI.settings.load()
      if (res) settings.value = res
      applyTheme(settings.value.app.theme || 'light')
    }, 'Failed to load settings')

  const saveSettings = () => withLoading(() => TauriAPI.settings.save(settings.value), 'Failed to save settings')

  async function saveSettingsWithUpdatePort(): Promise<boolean> {
    try {
      await TauriAPI.settings.saveWithUpdatePort(settings.value)
      return true
    } catch (err) {
      error.value = 'Failed to save settings'
      console.error('Failed to save settings:', err)
      return false
    }
  }

  const resetSettings = () =>
    withLoading(async () => {
      const res = await TauriAPI.settings.reset()
      if (res) settings.value = res
    }, 'Failed to reset settings')

  async function loadMountInfos() {
    try {
      mountInfos.value = await TauriAPI.rclone.mounts.list()
    } catch (err: any) {
      error.value = 'Failed to load mount information'
      console.error('Failed to load mount infos:', err)
    }
  }

  async function createRemoteConfig(name: string, type: string, config: RcloneFormConfig) {
    try {
      loading.value = true
      const fullConfig = {
        name,
        type,
        url: config.url,
        vendor: config.vendor || undefined,
        user: config.user,
        pass: config.pass,
        mountPoint: config.mountPoint || undefined,
        volumeName: config.volumeName || undefined,
        extraFlags: config.extraFlags || [],
        autoMount: config.autoMount ?? false
      }
      const createdConfig: RcloneWebdavConfig = {
        url: fullConfig.url,
        vendor: fullConfig.vendor || undefined,
        user: fullConfig.user,
        pass: fullConfig.pass
      }
      const result = await TauriAPI.rclone.remotes.create(name, type, createdConfig)
      if (!result) {
        throw new Error('Failed to create remote configuration')
      }
      settings.value.rclone.config[name] = fullConfig
      await loadRemoteConfigs()
      await saveSettings()
      return true
    } catch (err: any) {
      error.value = 'Failed to create remote configuration'
      console.error('Failed to create remote config:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function updateRemoteConfig(name: string, type: string, config: RcloneFormConfig) {
    try {
      loading.value = true
      const fullConfig = {
        name: config.name,
        type,
        url: config.url,
        vendor: config.vendor || undefined,
        user: config.user,
        pass: config.pass,
        mountPoint: config.mountPoint || undefined,
        volumeName: config.volumeName || undefined,
        extraFlags: config.extraFlags || [],
        autoMount: config.autoMount ?? false
      }
      const updatedConfig: RcloneWebdavConfig = {
        url: fullConfig.url,
        vendor: fullConfig.vendor || undefined,
        user: fullConfig.user,
        pass: fullConfig.pass
      }
      if (name !== config.name) {
        const result = createRemoteConfig(config.name, type, config)
        if (!result) {
          throw new Error('Failed to create remote configuration')
        }
        const deleteResult = await TauriAPI.rclone.remotes.delete(name)
        if (!deleteResult) {
          throw new Error('Failed to delete old remote configuration')
        }
      } else {
        const result = await TauriAPI.rclone.remotes.update(name, type, updatedConfig)
        if (!result) {
          throw new Error('Failed to update remote configuration')
        }
        settings.value.rclone.config[config.name] = fullConfig
      }
      await loadRemoteConfigs()
      if (name !== config.name && settings.value.rclone.config[name]) {
        delete settings.value.rclone.config[name]
      }
      const oldProcessId = await getRcloneMountProcessId(name)
      if (oldProcessId) {
        try {
          await TauriAPI.process.stop(oldProcessId)
          await TauriAPI.process.delete(oldProcessId)

          const mountArgs = [
            `${fullConfig.name}:${fullConfig.volumeName || ''}`,
            fullConfig.mountPoint || '',
            ...(fullConfig.extraFlags || [])
          ]
          const newProcessConfig: ProcessConfig = {
            id: `rclone_mount_${fullConfig.name}_process`,
            name: `rclone_mount_${fullConfig.name}_process`,
            args: mountArgs,
            auto_start: fullConfig.autoMount,
            bin_path: 'rclone',
            log_file: '',
            auto_restart: true,
            run_as_admin: false,
            created_at: 0,
            updated_at: 0
          }
          await TauriAPI.rclone.mounts.createProcess(newProcessConfig)
        } catch (err) {
          console.warn(`Failed to update mount process for renamed config ${name} -> ${config.name}:`, err)
        }
      }
      await saveSettings()
      await loadMountInfos()
      return true
    } catch (err: any) {
      error.value = 'Failed to update remote configuration'
      console.error('Failed to update remote config:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function deleteRemoteConfig(name: string) {
    try {
      loading.value = true
      const processId = await getRcloneMountProcessId(name)
      if (processId) {
        try {
          await TauriAPI.process.stop(processId)
          await TauriAPI.process.delete(processId)
        } catch (err) {
          console.warn(`Failed to stop/delete mount process for ${name}:`, err)
        }
      }
      await TauriAPI.rclone.remotes.delete(name)
      await loadRemoteConfigs()
      if (settings.value.rclone.config[name]) {
        delete settings.value.rclone.config[name]
        await saveSettings()
      }
      await loadMountInfos()
      return true
    } catch (err: any) {
      error.value = 'Failed to delete remote configuration'
      console.error('Failed to delete remote config:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function loadRemoteConfigs() {
    try {
      loading.value = true
      remoteConfigs.value = await TauriAPI.rclone.remotes.listConfig('webdav')
    } catch (err: any) {
      error.value = 'Failed to load remote configurations'
      console.error('Failed to load remote configs:', err)
    } finally {
      loading.value = false
    }
  }

  function getFullRcloneConfigs(name?: string): RcloneFormConfig[] {
    return name ? fullRcloneConfigs.value.filter(c => c.name === name) : fullRcloneConfigs.value
  }

  async function mountRemote(name: string) {
    try {
      loading.value = true
      const config = settings.value.rclone.config[name] as RcloneFormConfig | undefined
      if (!config) {
        throw new Error(`No configuration found for remote: ${name}`)
      }
      const processId = await getRcloneMountProcessId(name)
      console.log(`Mounting remote ${name} with process ID:`, processId)
      if (processId) {
        if (!config.mountPoint) {
          throw new Error(`Mount point is not set for remote: ${name}`)
        }
        const mountResult = await TauriAPI.rclone.mounts.check(config.mountPoint)
        if (!mountResult) {
          const startResult = await TauriAPI.process.start(processId)
          if (!startResult) {
            throw new Error(`Failed to start mount process for remote: ${name}`)
          }
          await loadMountInfos()
          return
        } else {
          console.log(`Remote ${name} is already mounted`)
          return
        }
      } else {
        const mountArgs = [
          `${config.name}:${config.volumeName || ''}`,
          config.mountPoint || '',
          ...(config.extraFlags || [])
        ]
        const createRemoteConfig: ProcessConfig = {
          id: `rclone_mount_${name}_process`,
          name: `rclone_mount_${name}_process`,
          args: mountArgs,
          auto_start: config.autoMount,
          bin_path: 'rclone',
          log_file: '',
          auto_restart: true,
          run_as_admin: false,
          created_at: 0,
          updated_at: 0
        }
        const createResponse = await TauriAPI.rclone.mounts.createProcess(createRemoteConfig)
        if (!createResponse || !createResponse.id) {
          throw new Error('Failed to create mount process')
        }
        const startResponse = await TauriAPI.process.start(createResponse.id)
        if (!startResponse) {
          throw new Error('Failed to start mount process')
        }
        await loadMountInfos()
      }
    } catch (err: any) {
      error.value = `Failed to mount remote ${name}: ${formatError(err)}`
      console.error('Failed to mount remote:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function unmountRemote(name: string) {
    try {
      loading.value = true
      const processId = await getRcloneMountProcessId(name)
      if (processId) {
        const stopResult = await TauriAPI.process.stop(processId)
        if (!stopResult) {
          throw new Error(`Failed to stop mount process for remote: ${name}`)
        }
      }
      await loadMountInfos()
    } catch (err: any) {
      error.value = `Failed to unmount remote ${name}: ${formatError(err)}`
      console.error('Failed to unmount remote:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  // Add update status tracking
  const updateAvailable = ref(false)
  const updateCheck = ref<UpdateCheck | null>(null)

  const openlistProcessId = ref<string | undefined>(undefined)

  const showTutorial = ref(false)
  const tutorialStep = ref(0)
  const tutorialSkipped = ref(false)

  async function getRcloneMountProcessId(name: string): Promise<string | undefined> {
    try {
      const processList = await TauriAPI.process.list()
      const mountName = `rclone_mount_${name}_process`
      const findRcloneBackend = processList.find(p => p.config?.name === mountName)
      if (findRcloneBackend) {
        return findRcloneBackend.id
      }
    } catch (err) {
      console.error('Failed to get Rclone process ID from database:', err)
      return undefined
    }
  }

  async function startOpenListCore() {
    try {
      loading.value = true

      let processId: string | undefined
      let createResponse: ProcessConfig | undefined
      const processList = await TauriAPI.process.list()
      const findOpenListCore = processList.find(p => p.config?.name === 'single_openlist_core_process')

      if (!findOpenListCore) {
        createResponse = await TauriAPI.core.create(settings.value.openlist.auto_launch)

        if (!createResponse || !createResponse.id) {
          throw new Error('Invalid response from TauriAPI.core.create: missing process ID')
        }

        processId = createResponse.id
      } else {
        processId = findOpenListCore.id
      }

      if (!processId) {
        throw new Error('Failed to create or retrieve OpenList Core process ID')
      }
      const startResponse = await TauriAPI.process.start(processId)
      if (!startResponse) {
        throw new Error('Failed to start OpenList Core service - service returned false')
      }

      openlistProcessId.value = processId
      await refreshOpenListCoreStatus()

      await TauriAPI.tray.update(openlistCoreStatus.value.running)
    } catch (err: any) {
      openlistCoreStatus.value = { running: false }
      let errorMessage = 'Failed to start service'
      const formattedError = formatError(err)
      if (formattedError) {
        errorMessage += `: ${formattedError}`
      }
      error.value = errorMessage
      console.error('Failed to start service:', err)
      await safeUpdateTrayMenu(false)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function getOpenListProcessId() {
    try {
      const processList = await TauriAPI.process.list()
      const findOpenListCore = processList.find(p => p.config?.name === 'single_openlist_core_process')
      if (findOpenListCore) {
        return findOpenListCore.id
      }
    } catch (err) {
      console.error('Failed to get OpenList process ID from database:', err)
      return undefined
    }
  }

  async function stopOpenListCore() {
    try {
      loading.value = true
      const id = await getOpenListProcessId()
      if (!id) {
        openlistCoreStatus.value = { running: false }
        await TauriAPI.tray.update(false)
        return
      }

      const result = await TauriAPI.process.stop(id)
      if (!result) {
        throw new Error('Failed to stop OpenList Core service - service returned false')
      }

      openlistCoreStatus.value = { running: false }
      await TauriAPI.tray.update(false)
    } catch (err: any) {
      const errorMessage = `Failed to stop service: ${formatError(err)}`
      error.value = errorMessage
      console.error('Failed to stop service:', err)
      try {
        await refreshOpenListCoreStatus()
      } catch (refreshErr) {
        console.error('Failed to refresh service status after stop failure:', refreshErr)
      }
      throw err
    } finally {
      loading.value = false
    }
  }

  async function enableAutoLaunch(autoLaunch: boolean) {
    try {
      const id = await getOpenListProcessId()
      if (!id) return
      const result = await TauriAPI.process.update(id, {
        auto_start: autoLaunch
      })
      if (!result) {
        throw new Error('Failed to enable auto-launch')
      }
    } catch (err) {
      const errorMessage = `Failed to enable auto-launch: ${formatError(err)}`
      error.value = errorMessage
      console.error('Failed to enable auto-launch:', err)
      throw err
    }
  }

  async function restartOpenListCore() {
    try {
      loading.value = true
      const id = await getOpenListProcessId()
      if (!id) {
        openlistCoreStatus.value = { running: false }
        await TauriAPI.tray.update(false)
        return
      }
      const result = await TauriAPI.process.restart(id)
      if (!result) {
        throw new Error('Failed to restart OpenList Core - service returned false')
      }
      await refreshOpenListCoreStatus()
      await TauriAPI.tray.update(openlistCoreStatus.value.running)
    } catch (err: any) {
      const errorMessage = `Failed to restart core: ${formatError(err)}`
      error.value = errorMessage
      console.error('Failed to restart core:', err)
      try {
        await refreshOpenListCoreStatus()
        await safeUpdateTrayMenu(openlistCoreStatus.value.running)
      } catch (refreshErr) {
        console.error('Failed to refresh core status after restart failure:', refreshErr)
      }
      throw err
    } finally {
      loading.value = false
    }
  }

  async function refreshOpenListCoreStatus() {
    try {
      const status = await TauriAPI.core.getStatus()
      const statusChanged = openlistCoreStatus.value.running !== status.running
      openlistCoreStatus.value = status
      if (statusChanged) {
        await TauriAPI.tray.updateDelayed(status.running)
      }
    } catch (err) {
      const wasRunning = openlistCoreStatus.value.running
      openlistCoreStatus.value = { running: false }
      if (wasRunning) {
        await TauriAPI.tray.updateDelayed(false)
      }
    }
  }

  async function loadLogs(source?: 'openlist' | 'rclone' | 'app') {
    try {
      source = source || 'openlist'
      const logEntries = await TauriAPI.logs.get(source)
      logs.value = logEntries
    } catch (err) {
      console.error('Failed to load logs:', err)
    }
  }

  async function clearLogs(source?: 'openlist' | 'rclone' | 'app') {
    try {
      loading.value = true
      const result = await TauriAPI.logs.clear(source)
      if (result) {
        logs.value = []
      } else {
        throw new Error('Failed to clear logs - backend returned false')
      }
    } catch (err) {
      error.value = 'Failed to clear logs'
      throw err
    } finally {
      loading.value = false
    }
  }

  const listFiles = (path: string) =>
    withLoading(async () => {
      files.value = await TauriAPI.files.list(path)
      currentPath.value = path
    }, 'Failed to list files')

  async function openFile(path: string) {
    try {
      await TauriAPI.files.open(path)
    } catch (err) {
      error.value = 'Failed to open file'
      console.error('Failed to open file:', err)
    }
  }

  async function openFolder(path: string) {
    try {
      await TauriAPI.files.folder(path)
    } catch (err) {
      error.value = 'Failed to open folder'
      console.error('Failed to open folder:', err)
    }
  }

  async function selectDirectory(title: string): Promise<string | null> {
    try {
      return await TauriAPI.util.selectDirectory(title)
    } catch (err) {
      console.error('Failed to select directory:', err)
      return null
    }
  }

  function clearError() {
    error.value = undefined
  }

  function formatError(err: any): string {
    if (err?.message) {
      return err.message
    } else if (typeof err === 'string') {
      return err
    } else if (err?.toString) {
      return err.toString()
    }
    return 'Unknown error occurred'
  }

  async function safeUpdateTrayMenu(running: boolean) {
    try {
      await TauriAPI.tray.update(running)
    } catch (err) {
      console.warn('Failed to update tray menu:', err)
    }
  }

  async function autoStartCoreIfEnabled() {
    try {
      if (settings.value.openlist.auto_launch) {
        await startOpenListCore()
      }
    } catch (err) {
      console.warn('Failed to auto-start core:', err)
    }
  }

  function applyTheme(theme: string) {
    const root = document.documentElement
    root.classList.remove('light', 'dark', 'auto')

    if (theme === 'auto') {
      root.classList.add('auto')
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
      root.classList.add(prefersDark ? 'dark' : 'light')

      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      mediaQuery.addEventListener('change', e => {
        if (settings.value.app.theme === 'auto') {
          root.classList.remove('light', 'dark')
          root.classList.add(e.matches ? 'dark' : 'light')
        }
      })
    } else {
      root.classList.add(theme)
    }
  }

  function setTheme(theme: 'light' | 'dark' | 'auto') {
    settings.value.app.theme = theme
    applyTheme(theme)
    saveSettings().catch(console.error)
  }

  function toggleTheme() {
    const currentTheme = settings.value.app.theme || 'light'
    const themes = ['light', 'dark', 'auto'] as const
    const currentIndex = themes.indexOf(currentTheme as any)
    const nextTheme = themes[(currentIndex + 1) % themes.length]
    setTheme(nextTheme)
  }

  async function init() {
    try {
      initTutorial()
      await loadSettings()
      await refreshOpenListCoreStatus()
      await TauriAPI.tray.updateDelayed(openlistCoreStatus.value.running)
      loadLogs()
      autoStartCoreIfEnabled()
      await loadRemoteConfigs()
      await loadMountInfos()
    } catch (err) {
      console.error('Application initialization failed:', err)
      throw err
    }
  }

  function initTutorial() {
    const hasSeenTutorial = localStorage.getItem('openlist-tutorial-completed')
    const tutorialDisabled = localStorage.getItem('openlist-tutorial-disabled')

    if (!hasSeenTutorial && tutorialDisabled !== 'true') {
      showTutorial.value = true
      tutorialStep.value = 0
    }
  }

  function startTutorial() {
    showTutorial.value = true
    tutorialStep.value = 0
    localStorage.removeItem('openlist-tutorial-disabled')
  }

  function nextTutorialStep() {
    tutorialStep.value++
  }

  function prevTutorialStep() {
    if (tutorialStep.value > 0) {
      tutorialStep.value--
    }
  }

  function skipTutorial() {
    showTutorial.value = false
    tutorialSkipped.value = true
    localStorage.setItem('openlist-tutorial-disabled', 'true')
  }

  function completeTutorial() {
    showTutorial.value = false
    localStorage.setItem('openlist-tutorial-completed', 'true')
  }

  async function getAdminPassword(): Promise<string | null> {
    try {
      return await TauriAPI.logs.adminPassword()
    } catch (err) {
      console.error('Failed to get admin password:', err)
      return null
    }
  }

  function closeTutorial() {
    showTutorial.value = false
  }

  // Update management functions
  function setUpdateAvailable(available: boolean, updateInfo?: UpdateCheck) {
    updateAvailable.value = available
    updateCheck.value = updateInfo || null
  }

  function clearUpdateStatus() {
    updateAvailable.value = false
    updateCheck.value = null
  }

  return {
    remoteConfigs,
    mountInfos,
    mountedConfigs,
    mountRemote,
    unmountRemote,
    createRemoteConfig,
    updateRemoteConfig,
    loadRemoteConfigs,
    defaultRcloneFormConfig,
    loadMountInfos,
    deleteRemoteConfig,
    getFullRcloneConfigs,
    fullRcloneConfigs,

    settings,
    openlistCoreStatus,
    logs,
    files,
    currentPath,
    loading,
    error,
    updateAvailable,
    updateCheck,

    showTutorial,
    tutorialStep,
    tutorialSkipped,

    isCoreRunning,
    openListCoreUrl,

    loadSettings,
    saveSettings,
    saveSettingsWithUpdatePort,
    resetSettings,

    startOpenListCore,
    stopOpenListCore,
    restartOpenListCore,
    enableAutoLaunch,
    refreshOpenListCoreStatus,
    loadLogs,
    clearLogs,
    listFiles,
    openFile,
    openFolder,
    selectDirectory,
    clearError,
    init,
    getAdminPassword,

    setTheme,
    toggleTheme,
    applyTheme,

    initTutorial,
    startTutorial,
    nextTutorialStep,
    prevTutorialStep,
    skipTutorial,
    completeTutorial,
    closeTutorial,
    setUpdateAvailable,
    clearUpdateStatus
  }
})
