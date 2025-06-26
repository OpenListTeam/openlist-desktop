<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, ComputedRef, Ref } from 'vue'
import { useTranslation } from '../composables/useI18n'
import { useRcloneStore } from '../stores/rclone'
import {
  HardDrive,
  Plus,
  Edit,
  Trash2,
  Play,
  Square,
  CheckCircle,
  XCircle,
  Loader,
  Cloud,
  Search,
  RefreshCw,
  Save,
  X,
  Settings,
  FolderOpen
} from 'lucide-vue-next'
import type { RcloneFormConfig } from '../types'
import { useAppStore } from '@/stores/app'

const { t } = useTranslation()
const rcloneStore = useRcloneStore()
const store = useAppStore()

const showAddForm = ref(false)
const editingConfig = ref<RcloneFormConfig | null>(null)
const searchQuery = ref('')
const statusFilter = ref<'all' | 'mounted' | 'unmounted' | 'error'>('all')

let mountRefreshInterval: NodeJS.Timeout | null = null
let backendStatusCheckInterval: NodeJS.Timeout | null = null

const configForm = ref({
  name: '',
  type: 'webdav',
  url: '',
  vendor: '',
  user: '',
  pass: '',
  mountPoint: '',
  volumeName: '',
  autoMount: false,
  extraFlags: [] as string[],
  extraOptions: {
    'vfs-cache-mode': 'full'
  }
}) as Ref<RcloneFormConfig>

const filteredConfigs: ComputedRef<RcloneFormConfig[]> = computed(() => {
  let filtered: RcloneFormConfig[] = []
  const fullRemoteConfigs = store.fullRcloneConfigs

  for (const config of fullRemoteConfigs) {
    if (!config) continue

    const matchesSearch = searchQuery.value
      ? config.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
        config.url.toLowerCase().includes(searchQuery.value.toLowerCase())
      : true
    if (!matchesSearch) continue

    const mountInfo = store.mountInfos.find(mount => mount.name === config.name)
    const status = mountInfo?.status || 'unmounted'
    const matchesStatus = statusFilter.value === 'all' || status === statusFilter.value

    if (matchesStatus && matchesSearch) {
      filtered.push(config)
    }
  }
  return filtered
})

const configCounts = computed(() => {
  const fullConfigs = store.fullRcloneConfigs
  return {
    total: fullConfigs.length,
    mounted: store.mountedConfigs.length,
    unmounted: fullConfigs.length - store.mountedConfigs.length,
    error: store.mountInfos.filter(m => m.status === 'error').length
  }
})

const addNewConfig = () => {
  resetForm()
  showAddForm.value = true
}

const editConfig = (config: RcloneFormConfig) => {
  editingConfig.value = config
  configForm.value = {
    name: config.name,
    type: config.type,
    url: config.url,
    vendor: config.vendor || '',
    user: config.user,
    pass: config.pass,
    mountPoint: config.mountPoint || '',
    volumeName: config.volumeName || '',
    autoMount: config.autoMount,
    extraFlags: config.extraFlags || ['--vfs-cache-mode', 'full']
  }
  showAddForm.value = true
}

const saveConfig = async () => {
  if (!configForm.value.name || !configForm.value.url || !configForm.value.user || !configForm.value.pass) {
    console.error(t('mount.messages.fillRequiredFields'))
    return
  }

  try {
    if (editingConfig.value && editingConfig.value.name) {
      await store.updateRemoteConfig(editingConfig.value.name, configForm.value.type, {
        name: configForm.value.name,
        type: configForm.value.type,
        url: configForm.value.url,
        vendor: configForm.value.vendor || undefined,
        user: configForm.value.user,
        pass: configForm.value.pass,
        mountPoint: configForm.value.mountPoint || undefined,
        volumeName: configForm.value.volumeName || undefined,
        autoMount: configForm.value.autoMount,
        extraFlags: configForm.value.extraFlags
      })
    } else {
      await store.createRemoteConfig(configForm.value.name, configForm.value.type, {
        name: configForm.value.name,
        type: configForm.value.type,
        url: configForm.value.url,
        vendor: configForm.value.vendor || undefined,
        user: configForm.value.user,
        pass: configForm.value.pass,
        mountPoint: configForm.value.mountPoint || undefined,
        volumeName: configForm.value.volumeName || undefined,
        autoMount: configForm.value.autoMount,
        extraFlags: configForm.value.extraFlags
      })
    }
    showAddForm.value = false
    resetForm()
  } catch (error: any) {
    console.error(error.message || t('mount.messages.failedToSave'))
  }
}

const cancelForm = () => {
  showAddForm.value = false
  resetForm()
}

const resetForm = () => {
  configForm.value = {
    name: '',
    type: 'webdav',
    url: '',
    vendor: '',
    user: '',
    pass: '',
    mountPoint: '',
    volumeName: '',
    autoMount: false,
    extraFlags: ['--vfs-cache-mode', 'full']
  }
  editingConfig.value = null
}

const mountConfig = async (config: RcloneFormConfig) => {
  try {
    await store.mountRemote(config.name)
  } catch (error: any) {
    console.error(error.message || t('mount.messages.failedToMount'))
  }
}

const unmountConfig = async (config: RcloneFormConfig) => {
  if (!config.name) return
  try {
    await store.unmountRemote(config.name)
  } catch (error: any) {
    console.error(error.message || t('mount.messages.failedToUnmount'))
  }
}

const deleteConfig = async (config: RcloneFormConfig) => {
  if (!config.name) return
  if (confirm(t('mount.messages.confirmDelete', { name: config.name }))) {
    try {
      await store.deleteRemoteConfig(config.name)
    } catch (error: any) {
      console.error(error.message || t('mount.messages.failedToDelete'))
    }
  }
}

const startBackend = async () => {
  try {
    await rcloneStore.startRcloneBackend()
    await new Promise(resolve => setTimeout(resolve, 1000))
    await rcloneStore.checkRcloneBackendStatus()
    await store.loadRemoteConfigs()
    await store.loadMountInfos()
  } catch (error: any) {
    console.error(error.message || t('mount.messages.failedToStartService'))
  }
}

const stopBackend = async () => {
  try {
    await rcloneStore.stopRcloneBackend()
  } catch (error: any) {
    console.error(error.message || t('mount.messages.failedToStopService'))
  }
}

const getConfigStatus = (config: RcloneFormConfig) => {
  const mountInfo = store.mountInfos.find(mount => mount.name === config.name)
  return mountInfo?.status || 'unmounted'
}

const getStatusIcon = (status: string) => {
  switch (status) {
    case 'mounted':
      return CheckCircle
    case 'mounting':
    case 'unmounting':
      return Loader
    case 'error':
      return XCircle
    default:
      return Square
  }
}

const isConfigMounted = (config: RcloneFormConfig) => {
  const status = getConfigStatus(config)
  return status === 'mounted'
}

const isConfigMounting = (config: RcloneFormConfig) => {
  const status = getConfigStatus(config)
  return status === 'mounting' || status === 'unmounting'
}

const addFlag = () => {
  if (!configForm.value.extraFlags) {
    configForm.value.extraFlags = []
  }
  configForm.value.extraFlags.push('')
}

const removeFlag = (index: number) => {
  if (configForm.value.extraFlags) {
    configForm.value.extraFlags.splice(index, 1)
  }
}

const handleKeydown = (event: KeyboardEvent) => {
  const key = event.key
  const ctrl = event.ctrlKey

  if (ctrl && key === 'n') {
    event.preventDefault()
    addNewConfig()
  } else if (ctrl && key === 'r') {
    event.preventDefault()
    store.loadRemoteConfigs()
    store.loadMountInfos()
  } else if (key === 'Escape') {
    event.preventDefault()
    if (showAddForm.value) {
      cancelForm()
    }
  }
}

const openInFileExplorer = async (path?: string) => {
  if (!path) {
    console.warn('Mount point path is not available')
    return
  }
  const normalizedPath = path.trim()
  try {
    await store.openFolder(normalizedPath)
  } catch (error: any) {
    console.error('Failed to open mount point in file explorer:', error)
    const errorMessage = error.message || error.toString() || 'Unknown error'
    if (errorMessage.includes('does not exist')) {
      console.warn(`Mount point path does not exist: ${normalizedPath}`)
    } else {
      console.error(`Failed to open file explorer: ${errorMessage}`)
    }
  }
}

onMounted(async () => {
  document.addEventListener('keydown', handleKeydown)
  await rcloneStore.checkRcloneBackendStatus()
  await store.loadRemoteConfigs()
  await store.loadMountInfos()
  mountRefreshInterval = setInterval(store.loadMountInfos, (store.settings.app.monitor_interval || 5) * 1000)
  backendStatusCheckInterval = setInterval(() => {
    rcloneStore.checkRcloneBackendStatus()
  }, (store.settings.app.monitor_interval || 5) * 1000)
  await rcloneStore.init()
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  if (mountRefreshInterval) {
    clearInterval(mountRefreshInterval)
  }
  if (backendStatusCheckInterval) {
    clearInterval(backendStatusCheckInterval)
  }
})
</script>

<template>
  <div class="mount-view">
    <!-- Header Section -->
    <div class="mount-header">
      <div class="header-content">
        <div class="header-info">
          <div class="title-section">
            <HardDrive class="header-icon" />
            <h1 class="page-title">{{ t('mount.title') }}</h1>
          </div>
          <div class="stats-overview">
            <div class="stat-item">
              <span class="stat-value">{{ configCounts.total }}</span>
              <span class="stat-label">{{ t('mount.stats.total') }}</span>
            </div>
            <div class="stat-divider"></div>
            <div class="stat-item success">
              <span class="stat-value">{{ configCounts.mounted }}</span>
              <span class="stat-label">{{ t('mount.stats.mounted') }}</span>
            </div>
            <div class="stat-divider"></div>
            <div class="stat-item neutral">
              <span class="stat-value">{{ configCounts.unmounted }}</span>
              <span class="stat-label">{{ t('mount.stats.unmounted') }}</span>
            </div>
            <div v-if="configCounts.error > 0" class="stat-divider"></div>
            <div v-if="configCounts.error > 0" class="stat-item error">
              <span class="stat-value">{{ configCounts.error }}</span>
              <span class="stat-label">{{ t('mount.stats.error') }}</span>
            </div>
          </div>
        </div>

        <div class="header-actions">
          <div class="service-indicator" :class="{ active: rcloneStore.serviceRunning }">
            <div class="indicator-dot"></div>
            <span class="indicator-text">
              {{ rcloneStore.serviceRunning ? t('mount.service.running') : t('mount.service.stopped') }}
            </span>
            <button
              @click="rcloneStore.serviceRunning ? stopBackend() : startBackend()"
              :class="['service-toggle', { active: rcloneStore.serviceRunning }]"
              :disabled="rcloneStore.loading"
            >
              <component :is="rcloneStore.serviceRunning ? Square : Play" class="btn-icon" />
            </button>
          </div>
          <button @click="addNewConfig" class="primary-btn">
            <Plus class="btn-icon" />
            <span>{{ t('mount.actions.addRemote') }}</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Controls Section -->
    <div class="controls-section">
      <div class="search-container">
        <Search class="search-icon" />
        <input
          v-model="searchQuery"
          type="text"
          :placeholder="t('mount.filters.searchPlaceholder')"
          class="search-input"
        />
      </div>
      <div class="filter-controls">
        <select v-model="statusFilter" class="status-filter">
          <option value="all">{{ t('mount.filters.allStatus') }}</option>
          <option value="mounted">{{ t('mount.status.mounted') }}</option>
          <option value="unmounted">{{ t('mount.status.unmounted') }}</option>
          <option value="error">{{ t('mount.status.error') }}</option>
        </select>
        <button @click="store.loadMountInfos" class="refresh-btn" :disabled="rcloneStore.loading">
          <RefreshCw class="refresh-icon" :class="{ spinning: rcloneStore.loading }" />
        </button>
      </div>
    </div>
    <!-- Error Display -->
    <div v-if="rcloneStore.error" class="error-alert">
      <XCircle class="alert-icon" />
      <span class="alert-message">{{ rcloneStore.error }}</span>
      <button @click="rcloneStore.clearError" class="alert-close">
        <X class="close-icon" />
      </button>
    </div>

    <!-- Remote Configurations -->
    <div class="configs-container">
      <div v-if="filteredConfigs.length === 0" class="empty-state">
        <div class="empty-content">
          <Cloud class="empty-icon" />
          <h3 class="empty-title">{{ t('mount.empty.title') }}</h3>
          <p class="empty-description">{{ t('mount.empty.description') }}</p>
          <button @click="addNewConfig" class="empty-action-btn">
            <Plus class="btn-icon" />
            <span>{{ t('mount.actions.addRemote') }}</span>
          </button>
        </div>
      </div>

      <div v-else class="config-grid">
        <div
          v-for="config in filteredConfigs"
          :key="config.name"
          class="config-card"
          :class="{
            mounted: isConfigMounted(config),
            error: getConfigStatus(config) === 'error',
            loading: isConfigMounting(config)
          }"
        >
          <div class="card-header">
            <div class="config-info">
              <div class="config-icon">
                <Cloud />
              </div>
              <div class="config-details">
                <h3 class="config-name">{{ config.name }}</h3>
                <p class="config-url">{{ config.url }}</p>
              </div>
            </div>
            <div class="config-status">
              <component
                :is="getStatusIcon(getConfigStatus(config))"
                class="status-icon"
                :class="{
                  spinning: isConfigMounting(config) || store.loading,
                  success: getConfigStatus(config) === 'mounted',
                  error: getConfigStatus(config) === 'error'
                }"
              />
            </div>
          </div>

          <div class="card-meta">
            <div class="meta-tags">
              <span class="meta-tag">{{ config.type }}</span>
              <span
                v-if="config.mountPoint"
                class="meta-tag clickable-mount-point"
                @click="openInFileExplorer(config.mountPoint)"
                :title="t('mount.meta.openInExplorer')"
              >
                <FolderOpen class="mount-point-icon" />
                {{ config.mountPoint }}
              </span>
              <span v-if="config.volumeName" class="meta-tag">{{ config.volumeName }}</span>
              <span v-if="config.autoMount" class="meta-tag auto">{{ t('mount.meta.autoMount') }}</span>
            </div>
          </div>

          <div class="card-actions">
            <div class="action-group">
              <button
                v-if="!isConfigMounted(config)"
                @click="mountConfig(config)"
                class="action-btn primary"
                :disabled="isConfigMounting(config) || !config.mountPoint"
                :title="!config.mountPoint ? t('mount.messages.mountPointRequired') : ''"
              >
                <Play class="btn-icon" />
                <span>{{ t('mount.actions.mount') }}</span>
              </button>
              <button
                v-else
                @click="unmountConfig(config)"
                class="action-btn warning"
                :disabled="isConfigMounting(config)"
              >
                <Square class="btn-icon" />
                <span>{{ t('mount.actions.unmount') }}</span>
              </button>
            </div>

            <div class="secondary-actions">
              <button @click="editConfig(config)" class="secondary-btn" :title="t('mount.actions.edit')">
                <Edit class="btn-icon" />
              </button>
              <button
                @click="deleteConfig(config)"
                class="secondary-btn danger"
                :disabled="isConfigMounted(config)"
                :title="t('mount.actions.delete')"
              >
                <Trash2 class="btn-icon" />
              </button>
              <button
                v-if="isConfigMounted(config)"
                @click="openInFileExplorer(config.mountPoint)"
                class="secondary-btn"
                :title="t('mount.actions.openInExplorer')"
              >
                <FolderOpen class="btn-icon" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
    <!-- Configuration Modal -->
    <div v-if="showAddForm" class="modal-backdrop" @click="cancelForm">
      <div class="config-modal" @click.stop>
        <div class="modal-header">
          <div class="modal-title-section">
            <Settings class="modal-icon" />
            <h2 class="modal-title">
              {{ editingConfig ? t('mount.config.editTitle') : t('mount.config.addTitle') }}
            </h2>
          </div>
          <button @click="cancelForm" class="modal-close">
            <X class="close-icon" />
          </button>
        </div>

        <div class="modal-content">
          <div class="config-form">
            <div class="form-section">
              <h3 class="section-title">{{ t('mount.config.basicInfo') }}</h3>
              <div class="form-grid">
                <div class="form-field">
                  <label class="field-label">{{ t('mount.config.name') }} *</label>
                  <input
                    v-model="configForm.name"
                    type="text"
                    class="field-input"
                    :placeholder="t('mount.config.namePlaceholder')"
                    required
                  />
                </div>
                <div class="form-field">
                  <label class="field-label">{{ t('mount.config.type') }} *</label>
                  <select v-model="configForm.type" class="field-select" required>
                    <option value="webdav">{{ t('mount.config.types.webdav') }}</option>
                  </select>
                </div>
              </div>

              <div class="form-field">
                <label class="field-label">{{ t('mount.config.url') }} *</label>
                <input
                  v-model="configForm.url"
                  type="url"
                  class="field-input"
                  :placeholder="t('mount.config.urlPlaceholder')"
                  required
                />
              </div>

              <div v-if="configForm.type === 'webdav'" class="form-field">
                <label class="field-label">{{ t('mount.config.vendor') }}</label>
                <input
                  v-model="configForm.vendor"
                  type="text"
                  class="field-input"
                  :placeholder="t('mount.config.vendorPlaceholder')"
                />
              </div>
            </div>

            <div class="form-section">
              <h3 class="section-title">{{ t('mount.config.authentication') }}</h3>
              <div class="form-grid">
                <div class="form-field">
                  <label class="field-label">{{ t('mount.config.username') }} *</label>
                  <input
                    v-model="configForm.user"
                    type="text"
                    class="field-input"
                    :placeholder="t('mount.config.usernamePlaceholder')"
                    required
                  />
                </div>
                <div class="form-field">
                  <label class="field-label">{{ t('mount.config.password') }} *</label>
                  <input
                    v-model="configForm.pass"
                    type="text"
                    class="field-input"
                    :placeholder="t('mount.config.passwordPlaceholder')"
                    required
                  />
                </div>
              </div>
            </div>

            <div class="form-section">
              <h3 class="section-title">{{ t('mount.config.mountSettings') }}</h3>
              <div class="form-grid">
                <div class="form-field">
                  <label class="field-label">{{ t('mount.config.mountPoint') }}</label>
                  <input
                    v-model="configForm.mountPoint"
                    type="text"
                    class="field-input"
                    :placeholder="t('mount.config.mountPointPlaceholder')"
                  />
                </div>
                <div class="form-field">
                  <label class="field-label">{{ t('mount.config.volumeName') }}</label>
                  <input
                    v-model="configForm.volumeName"
                    type="text"
                    class="field-input"
                    :placeholder="t('mount.config.volumeNamePlaceholder')"
                  />
                </div>
              </div>

              <div class="form-field">
                <label class="checkbox-field">
                  <input v-model="configForm.autoMount" type="checkbox" class="checkbox-input" />
                  <span class="checkbox-label">{{ t('mount.config.autoMount') }}</span>
                </label>
              </div>
            </div>

            <div class="form-section">
              <h3 class="section-title">{{ t('mount.config.advancedSettings') }}</h3>
              <div class="form-field">
                <label class="field-label">{{ t('mount.config.extraFlags') }}</label>
                <div class="flags-container">
                  <div v-for="(_, index) in configForm.extraFlags || []" :key="index" class="flag-item">
                    <input
                      v-model="configForm.extraFlags![index]"
                      type="text"
                      class="flag-input"
                      :placeholder="t('mount.config.flagPlaceholder')"
                    />
                    <button
                      @click="removeFlag(index)"
                      type="button"
                      class="remove-flag-btn"
                      :title="t('mount.config.removeFlag')"
                    >
                      <X class="btn-icon" />
                    </button>
                  </div>
                  <button @click="addFlag" type="button" class="add-flag-btn">
                    <Plus class="btn-icon" />
                    <span>{{ t('mount.config.addFlag') }}</span>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="modal-footer">
          <button @click="cancelForm" class="cancel-btn">
            <X class="btn-icon" />
            <span>{{ t('common.cancel') }}</span>
          </button>
          <button @click="saveConfig" class="save-btn" :disabled="store.loading">
            <Save class="btn-icon" />
            <span>{{ editingConfig ? t('common.save') : t('common.add') }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped src="./css/MountView.css"></style>
