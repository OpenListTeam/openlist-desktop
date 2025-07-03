<template>
  <Card :title="t('dashboard.serviceManagement.title')" variant="elevated" hover class="service-management-card">
    <div class="service-sections">
      <div class="status-section">
        <div class="status-header">
          <div class="status-info">
            <h4>{{ t('dashboard.serviceManagement.serviceStatus') }}</h4>
            <div class="status-indicator" :class="statusClass">
              <div class="status-dot"></div>
              <span>{{ statusText }}</span>
            </div>
          </div>
          <div class="service-icon" :class="statusClass">
            <component :is="statusIcon" :size="24" />
          </div>
        </div>
      </div>

      <div class="actions-section">
        <div class="action-buttons">
          <button
            @click="installService"
            :disabled="actionLoading || serviceStatus === 'installed'"
            class="action-btn install-btn"
            v-if="serviceStatus !== 'running' && serviceStatus !== 'stopped'"
          >
            <component :is="actionLoading && currentAction === 'install' ? LoaderIcon : Download" :size="16" />
            <span>{{
              actionLoading && currentAction === 'install'
                ? t('common.loading')
                : t('dashboard.serviceManagement.install')
            }}</span>
          </button>

          <button
            @click="startService"
            :disabled="actionLoading || (serviceStatus !== 'installed' && serviceStatus !== 'stopped')"
            class="action-btn start-btn"
            v-if="serviceStatus === 'installed' || serviceStatus === 'stopped'"
          >
            <component :is="actionLoading && currentAction === 'start' ? LoaderIcon : Play" :size="16" />
            <span>{{
              actionLoading && currentAction === 'start' ? t('common.loading') : t('dashboard.serviceManagement.start')
            }}</span>
          </button>

          <button
            @click="stopService"
            :disabled="actionLoading"
            class="action-btn stop-btn"
            v-if="serviceStatus === 'running'"
          >
            <component :is="actionLoading && currentAction === 'stop' ? LoaderIcon : Stop" :size="16" />
            <span>{{
              actionLoading && currentAction === 'stop' ? t('common.loading') : t('dashboard.serviceManagement.stop')
            }}</span>
          </button>

          <button
            @click="showUninstallDialog = true"
            :disabled="actionLoading"
            class="action-btn uninstall-btn"
            v-if="serviceStatus !== 'not-installed'"
          >
            <component :is="actionLoading && currentAction === 'uninstall' ? LoaderIcon : Trash2" :size="16" />
            <span>{{
              actionLoading && currentAction === 'uninstall'
                ? t('common.loading')
                : t('dashboard.serviceManagement.uninstall')
            }}</span>
          </button>
        </div>
      </div>
    </div>

    <ConfirmDialog
      :is-open="showUninstallDialog"
      :title="t('dashboard.serviceManagement.confirmUninstall.title')"
      :message="t('dashboard.serviceManagement.confirmUninstall.message')"
      :confirm-text="t('dashboard.serviceManagement.confirmUninstall.confirm')"
      :cancel-text="t('common.cancel')"
      variant="danger"
      @confirm="confirmUninstall"
      @cancel="cancelUninstall"
    />
  </Card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useTranslation } from '../../composables/useI18n'
import {
  Download,
  Play,
  Square as Stop,
  Trash2,
  Loader2 as LoaderIcon,
  CheckCircle2,
  XCircle,
  Circle,
  Server
} from 'lucide-vue-next'
import Card from '../ui/Card.vue'
import ConfirmDialog from '../ui/ConfirmDialog.vue'
import { TauriAPI } from '../../api/tauri'
import { useRcloneStore } from '@/stores/rclone'
import { useAppStore } from '../../stores/app'

const store = useAppStore()

const { t } = useTranslation()
const rcloneStore = useRcloneStore()

const serviceStatus = ref<'not-installed' | 'installed' | 'running' | 'error' | 'stopped'>('not-installed')
const actionLoading = ref(false)
const currentAction = ref('')
const showUninstallDialog = ref(false)

let statusCheckInterval: number | null = null

const statusClass = computed(() => {
  switch (serviceStatus.value) {
    case 'running':
      return 'status-running'
    case 'installed':
      return 'status-installed'
    case 'error':
    case 'stopped':
      return 'status-error'
    default:
      return 'status-not-installed'
  }
})

const statusText = computed(() => {
  switch (serviceStatus.value) {
    case 'running':
      return t('dashboard.serviceManagement.status.running')
    case 'installed':
      return t('dashboard.serviceManagement.status.installed')
    case 'error':
      return t('dashboard.serviceManagement.status.error')
    case 'stopped':
      return t('dashboard.serviceManagement.status.stopped')
    default:
      return t('dashboard.serviceManagement.status.notInstalled')
  }
})

const statusIcon = computed(() => {
  switch (serviceStatus.value) {
    case 'running':
      return CheckCircle2
    case 'installed':
      return Circle
    case 'error':
      return XCircle
    case 'stopped':
      return Stop
    default:
      return Server
  }
})

const checkServiceStatus = async () => {
  try {
    const status = await TauriAPI.service.status()
    serviceStatus.value = status as 'not-installed' | 'installed' | 'running' | 'error' | 'stopped'
    return status
  } catch (error) {
    console.error('Failed to check service status:', error)
    serviceStatus.value = 'error'
    return 'error'
  }
}

const installService = async () => {
  actionLoading.value = true
  currentAction.value = 'install'
  try {
    const result = await TauriAPI.service.install()
    if (!result) {
      throw new Error('Installation failed')
    }
    await new Promise(resolve => setTimeout(resolve, 5000))
    const status = await checkServiceStatus()
    if (status !== 'installed' && status !== 'running' && status !== 'stopped') {
      throw new Error('Service installation did not complete successfully')
    }
    try {
      await TauriAPI.rclone.backend.createAndStart()
      await rcloneStore.checkRcloneBackendStatus()
    } catch (stopError) {
      console.warn('Failed to stop service during installation:', stopError)
    }
  } catch (error) {
    serviceStatus.value = 'error'
  } finally {
    actionLoading.value = false
    currentAction.value = ''
  }
}

const startService = async () => {
  actionLoading.value = true
  currentAction.value = 'start'
  try {
    const result = await TauriAPI.service.start()
    if (!result) {
      throw new Error('Service start failed')
    }
    serviceStatus.value = 'running'
  } catch (error) {
    console.error('Failed to start service:', error)
    serviceStatus.value = 'error'
  } finally {
    actionLoading.value = false
    currentAction.value = ''
  }
}

const stopService = async () => {
  actionLoading.value = true
  currentAction.value = 'stop'
  try {
    const result = await TauriAPI.service.stop()
    if (!result) {
      throw new Error('Service stop failed')
    }
    await checkServiceStatus()
  } catch (error) {
    console.error('Failed to stop service:', error)
    serviceStatus.value = 'error'
  } finally {
    actionLoading.value = false
    currentAction.value = ''
  }
}

const uninstallService = async () => {
  actionLoading.value = true
  currentAction.value = 'uninstall'
  try {
    const result = await TauriAPI.service.uninstall()
    if (!result) {
      throw new Error('Uninstallation failed')
    }
    serviceStatus.value = 'not-installed'
    await rcloneStore.checkRcloneBackendStatus()
  } catch (error) {
    console.error('Failed to uninstall service:', error)
    serviceStatus.value = 'error'
  } finally {
    actionLoading.value = false
    currentAction.value = ''
  }
}

const confirmUninstall = async () => {
  showUninstallDialog.value = false
  await uninstallService()
}

const cancelUninstall = () => {
  showUninstallDialog.value = false
}

onMounted(async () => {
  await checkServiceStatus()
  statusCheckInterval = window.setInterval(checkServiceStatus, (store.settings.app.monitor_interval || 5) * 1000)
})

onUnmounted(() => {
  if (statusCheckInterval) {
    clearInterval(statusCheckInterval)
  }
})
</script>

<style scoped>
.service-sections {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  flex: 1;
}

.status-section {
  border: 1px solid rgb(229 231 235);
  border-radius: 0.75rem;
  padding: 1.25rem;
  background: rgb(249 250 251);
}

:root.dark .status-section,
:root.auto.dark .status-section {
  border-color: rgb(55 65 81);
  background: rgb(31 41 55);
}

.status-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 1rem;
}

.status-info h4 {
  margin: 0 0 0.5rem 0;
  font-size: 1rem;
  font-weight: 600;
  color: rgb(17 24 39);
}

:root.dark .status-info h4,
:root.auto.dark .status-info h4 {
  color: rgb(243 244 246);
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
  font-weight: 500;
}

.status-dot {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 50%;
}

.status-running .status-dot {
  background: rgb(34 197 94);
  box-shadow: 0 0 0 2px rgba(34, 197, 94, 0.2);
}

.status-installed .status-dot {
  background: rgb(251 191 36);
  box-shadow: 0 0 0 2px rgba(251, 191, 36, 0.2);
}

.status-error .status-dot {
  background: rgb(239 68 68);
  box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.2);
}

.status-not-installed .status-dot {
  background: rgb(107 114 128);
  box-shadow: 0 0 0 2px rgba(107, 114, 128, 0.2);
}

.status-running {
  color: rgb(34 197 94);
}

.status-installed {
  color: rgb(251 191 36);
}

.status-error {
  color: rgb(239 68 68);
}

.status-not-installed {
  color: rgb(107 114 128);
}

.service-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3rem;
  height: 3rem;
  border-radius: 0.75rem;
  margin-left: 1rem;
}

.service-icon.status-running {
  background: linear-gradient(135deg, rgba(34, 197, 94, 0.1), rgba(34, 197, 94, 0.2));
  color: rgb(34 197 94);
}

.service-icon.status-installed {
  background: linear-gradient(135deg, rgba(251, 191, 36, 0.1), rgba(251, 191, 36, 0.2));
  color: rgb(251 191 36);
}

.service-icon.status-error {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.1), rgba(239, 68, 68, 0.2));
  color: rgb(239 68 68);
}

.service-icon.status-not-installed {
  background: linear-gradient(135deg, rgba(107, 114, 128, 0.1), rgba(107, 114, 128, 0.2));
  color: rgb(107 114 128);
}

.service-details {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

:root.dark .status-section,
:root.auto.dark .status-section {
  border-color: rgb(55 65 81);
  background: rgb(31 41 55);
}

.actions-section {
  border: 1px solid rgb(229 231 235);
  border-radius: 0.75rem;
  padding: 1.25rem;
  background: rgb(249 250 251);
}

:root.dark .actions-section,
:root.auto.dark .actions-section {
  border-color: rgb(55 65 81);
  background: rgb(31 41 55);
}

.action-buttons {
  display: flex;
  gap: 0.75rem;
  flex-wrap: wrap;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  font-size: 0.875rem;
  font-weight: 500;
  border: none;
  border-radius: 0.5rem;
  cursor: pointer;
  transition: all 0.2s;
  flex: 1;
  justify-content: center;
  min-width: 7rem;
}

.install-btn {
  background: rgb(34 197 94);
  color: white;
}

.install-btn:hover:not(:disabled) {
  background: rgb(21 128 61);
}

.start-btn {
  background: rgb(59 130 246);
  color: white;
}

.start-btn:hover:not(:disabled) {
  background: rgb(37 99 235);
}

.stop-btn {
  background: rgb(239 68 68);
  color: white;
}

.stop-btn:hover:not(:disabled) {
  background: rgb(220 38 38);
}

.restart-btn {
  background: rgb(251 191 36);
  color: white;
}

.restart-btn:hover:not(:disabled) {
  background: rgb(245 158 11);
}

.uninstall-btn {
  background: rgb(107 114 128);
  color: white;
}

.uninstall-btn:hover:not(:disabled) {
  background: rgb(75 85 99);
}

.action-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* Loading animation */
.action-btn [data-lucide='loader-2'],
.logs-refresh-btn [data-lucide='loader-2'] {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

/* Responsive design */
@media (max-width: 768px) {
  .action-buttons {
    flex-direction: column;
  }

  .action-btn {
    flex: none;
  }
}
</style>
