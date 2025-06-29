import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'

import { useAppStore } from '../stores/app'

export function useLogs() {
  const store = useAppStore()

  const logContainer = ref<HTMLElement>()
  const autoScroll = ref(true)
  const filterLevel = ref<string>('all')
  const filterSource = ref<string>('all')
  const searchQuery = ref('')
  const selectedLogEntry = ref<any>(null)

  let logRefreshInterval: NodeJS.Timeout | null = null

  const filteredLogs = computed(() => {
    let logs = store.logs || []

    if (filterLevel.value !== 'all') {
      logs = logs.filter((log: any) => log.level === filterLevel.value)
    }

    if (filterSource.value !== 'all') {
      logs = logs.filter((log: any) => log.source === filterSource.value)
    }

    if (searchQuery.value.trim()) {
      const query = searchQuery.value.toLowerCase()
      logs = logs.filter(
        (log: any) => log.message.toLowerCase().includes(query) || log.source.toLowerCase().includes(query)
      )
    }

    return logs.slice(-500)
  })

  const logLevelClass = (level: string) => {
    switch (level) {
      case 'error':
        return 'log-error'
      case 'warn':
        return 'log-warning'
      case 'info':
        return 'log-info'
      case 'debug':
        return 'log-debug'
      default:
        return 'log-info'
    }
  }

  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleTimeString()
  }

  const scrollToBottom = async () => {
    if (autoScroll.value && logContainer.value) {
      await nextTick()
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }
  }

  const clearLogs = async (source?: 'openlist' | 'rclone' | 'app') => {
    try {
      await store.clearLogs(source)
    } catch (error) {
      console.error('Failed to clear logs:', error)
      throw error
    }
  }

  const copyLogsToClipboard = async () => {
    const logsText = filteredLogs.value
      .map((log: any) => `[${log.timestamp}] [${log.level.toUpperCase()}] [${log.source}] ${log.message}`)
      .join('\n')

    try {
      await navigator.clipboard.writeText(logsText)
    } catch (error) {
      console.error('Failed to copy logs:', error)
    }
  }

  const exportLogs = () => {
    const logsText = filteredLogs.value
      .map((log: any) => `[${log.timestamp}] [${log.level.toUpperCase()}] [${log.source}] ${log.message}`)
      .join('\n')

    const blob = new Blob([logsText], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `openlist-logs-${new Date().toISOString().split('T')[0]}.txt`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  const startLogRefresh = (interval = 2000) => {
    if (logRefreshInterval) return

    logRefreshInterval = setInterval(async () => {
      const oldLength = store.logs?.length || 0
      await store.loadLogs()
      if (store.logs?.length > oldLength) {
        await scrollToBottom()
      }
    }, interval)
  }

  const stopLogRefresh = () => {
    if (logRefreshInterval) {
      clearInterval(logRefreshInterval)
      logRefreshInterval = null
    }
  }

  onMounted(async () => {
    await store.loadLogs()
    await scrollToBottom()
    startLogRefresh()
  })

  onUnmounted(() => {
    stopLogRefresh()
  })

  return {
    logContainer,
    autoScroll,
    filterLevel,
    filterSource,
    searchQuery,
    selectedLogEntry,
    filteredLogs,
    logLevelClass,
    formatTimestamp,
    scrollToBottom,
    clearLogs,
    copyLogsToClipboard,
    exportLogs,
    startLogRefresh,
    stopLogRefresh
  }
}
