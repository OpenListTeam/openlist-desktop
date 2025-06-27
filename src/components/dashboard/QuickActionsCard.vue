<template>
  <Card :title="t('dashboard.quickActions.title')" variant="elevated" hover class="quick-actions-card">
    <div class="actions-grid">
      <div class="action-section">
        <div class="section-header">
          <h4>{{ t('dashboard.quickActions.openlistService') }}</h4>
        </div>
        <div class="action-buttons">
          <button
            @click="toggleCore"
            :class="['action-btn', 'service-btn', { running: isCoreRunning }]"
            :disabled="loading"
          >
            <component :is="serviceButtonIcon" :size="20" />
            <span>{{ serviceButtonText }}</span>
          </button>

          <button @click="restartCore" :disabled="!isCoreRunning || loading" class="action-btn restart-btn">
            <RotateCcw :size="18" />
            <span>{{ t('dashboard.quickActions.restart') }}</span>
          </button>

          <button @click="openWebUI" :disabled="!isCoreRunning" class="action-btn web-btn">
            <ExternalLink :size="18" />
            <span>{{ t('dashboard.quickActions.openWeb') }}</span>
          </button>

          <button
            @click="showAdminPassword"
            class="action-btn password-btn icon-only-btn"
            :title="t('dashboard.quickActions.showAdminPassword')"
          >
            <Key :size="16" />
          </button>
        </div>
      </div>

      <div class="action-section">
        <div class="section-header">
          <h4>{{ t('dashboard.quickActions.rclone') }}</h4>
        </div>
        <div class="action-buttons">
          <button
            @click="rcloneStore.serviceRunning ? rcloneStore.stopRcloneBackend() : rcloneStore.startRcloneBackend()"
            :disabled="loading || rcloneStore.loading"
            :class="['action-btn', 'service-indicator-btn', { active: rcloneStore.serviceRunning }]"
          >
            <component :is="rcloneStore.serviceRunning ? Square : Play" :size="18" />
            <span>{{
              rcloneStore.serviceRunning
                ? t('dashboard.quickActions.stopRclone')
                : t('dashboard.quickActions.startRclone')
            }}</span>
          </button>

          <button @click="openRcloneConfig" class="action-btn config-btn">
            <Settings :size="18" />
            <span>{{ t('dashboard.quickActions.configRclone') }}</span>
          </button>

          <button @click="viewMounts" class="action-btn mount-btn">
            <HardDrive :size="18" />
            <span>{{ t('dashboard.quickActions.manageMounts') }}</span>
          </button>
        </div>
      </div>

      <!-- Quick Settings -->
      <div class="action-section">
        <div class="section-header">
          <h4>{{ t('dashboard.quickActions.quickSettings') }}</h4>
        </div>
        <div class="settings-toggles">
          <label class="toggle-item">
            <input type="checkbox" v-model="settings.openlist.auto_launch" @change="handleAutoLaunchToggle" />
            <span class="toggle-text">{{ t('dashboard.quickActions.autoLaunch') }}</span>
          </label>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '../../stores/app'
import { useRcloneStore } from '../../stores/rclone'
import { useTranslation } from '../../composables/useI18n'
import Card from '../ui/Card.vue'
import { Play, Square, RotateCcw, ExternalLink, Settings, HardDrive, Loader, Key } from 'lucide-vue-next'

const { t } = useTranslation()
const router = useRouter()
const store = useAppStore()
const rcloneStore = useRcloneStore()

const isCoreRunning = computed(() => store.isCoreRunning)
const loading = computed(() => store.loading)
const settings = computed(() => store.settings)
let statusCheckInterval: number | null = null

const serviceButtonIcon = computed(() => {
  if (loading.value) return Loader
  return isCoreRunning.value ? Square : Play
})

const serviceButtonText = computed(() => {
  if (loading.value) return t('dashboard.quickActions.processing')
  return isCoreRunning.value
    ? t('dashboard.quickActions.stopOpenListCore')
    : t('dashboard.quickActions.startOpenListCore')
})

const toggleCore = async () => {
  if (isCoreRunning.value) {
    await store.stopOpenListCore()
  } else {
    await store.startOpenListCore()
  }
}

const restartCore = async () => {
  await store.restartOpenListCore()
}

const openWebUI = () => {
  if (store.openListCoreUrl) {
    window.open(store.openListCoreUrl, '_blank')
  }
}

const openRcloneConfig = () => {
  router.push({ name: 'Settings', query: { tab: 'rclone' } })
}

const viewMounts = () => {
  router.push({ name: 'Mount' })
}

const showAdminPassword = async () => {
  try {
    const password = await store.getAdminPassword()
    if (password) {
      await navigator.clipboard.writeText(password)

      const notification = document.createElement('div')
      notification.innerHTML = `
        <div style="
          position: fixed;
          top: 20px;
          right: 20px;
          background: linear-gradient(135deg, rgb(16, 185, 129), rgb(5, 150, 105));
          color: white;
          padding: 12px 20px;
          border-radius: 8px;
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
          z-index: 10000;
          font-weight: 500;
          max-width: 300px;
          word-break: break-all;
        ">
          <div style="display: flex; align-items: center; gap: 8px;">
            <div style="font-size: 18px;">✓</div>
            <div>
              <div style="font-size: 14px; margin-bottom: 4px;">Admin password copied!</div>
              <div style="font-size: 12px; opacity: 0.9; font-family: monospace;">${password}</div>
            </div>
          </div>
        </div>
      `
      document.body.appendChild(notification)

      setTimeout(() => {
        if (notification.parentNode) {
          notification.parentNode.removeChild(notification)
        }
      }, 4000)
    } else {
      const notification = document.createElement('div')
      notification.innerHTML = `
        <div style="
          position: fixed;
          top: 20px;
          right: 20px;
          background: linear-gradient(135deg, rgb(239, 68, 68), rgb(220, 38, 38));
          color: white;
          padding: 12px 20px;
          border-radius: 8px;
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
          z-index: 10000;
          font-weight: 500;
          max-width: 350px;
        ">
          <div style="display: flex; align-items: center; gap: 8px;">
            <div style="font-size: 18px;">⚠</div>
            <div>
              <div style="font-size: 14px; margin-bottom: 4px;">No admin password found</div>
              <div style="font-size: 12px; opacity: 0.9;">Make sure OpenList Core has been started at least once.</div>
            </div>
          </div>
        </div>
      `
      document.body.appendChild(notification)

      setTimeout(() => {
        if (notification.parentNode) {
          notification.parentNode.removeChild(notification)
        }
      }, 4000)
    }
  } catch (error) {
    console.error('Failed to get admin password:', error)

    const notification = document.createElement('div')
    notification.innerHTML = `
      <div style="
        position: fixed;
        top: 20px;
        right: 20px;
        background: linear-gradient(135deg, rgb(239, 68, 68), rgb(220, 38, 38));
        color: white;
        padding: 12px 20px;
        border-radius: 8px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        z-index: 10000;
        font-weight: 500;
        max-width: 300px;
      ">
        <div style="display: flex; align-items: center; gap: 8px;">
          <div style="font-size: 18px;">✗</div>
          <div>
            <div style="font-size: 14px; margin-bottom: 4px;">Failed to get admin password</div>
            <div style="font-size: 12px; opacity: 0.9;">Please check the logs.</div>
          </div>
        </div>
      </div>
    `
    document.body.appendChild(notification)

    setTimeout(() => {
      if (notification.parentNode) {
        notification.parentNode.removeChild(notification)
      }
    }, 4000)
  }
}

const handleAutoLaunchToggle = () => {
  store.enableAutoLaunch(settings.value.openlist.auto_launch)
  saveSettings()
}

const saveSettings = async () => {
  await store.saveSettings()
}

onMounted(async () => {
  await rcloneStore.checkRcloneBackendStatus()
  statusCheckInterval = window.setInterval(
    rcloneStore.checkRcloneBackendStatus,
    (store.settings.app.monitor_interval || 5) * 1000
  )
})

onUnmounted(() => {
  if (statusCheckInterval) {
    clearInterval(statusCheckInterval)
  }
})
</script>

<style scoped>
.actions-grid {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.action-section {
  position: relative;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.75rem;
}

.section-header h4 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  letter-spacing: -0.025em;
}

.icon-only-btn {
  flex: 0 0 auto;
  min-width: auto;
  width: 1.75rem;
  padding: 0.375rem;
  justify-content: center;
}

.icon-only-btn span {
  display: none;
}

.action-buttons {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  gap: 0.5rem;
  align-items: stretch;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  border: 1px solid var(--color-border-secondary);
  border-radius: 10px;
  background: var(--color-surface);
  backdrop-filter: blur(10px);
  color: var(--color-text-primary);
  font-size: 0.8125rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: var(--shadow-sm);
  flex: 1;
  min-width: 0;
  text-align: center;
  justify-content: center;
}

.action-btn span {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.action-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  background: var(--color-surface-elevated);
  border-color: rgba(59, 130, 246, 0.3);
  box-shadow: var(--shadow-md);
}

.action-btn:active {
  transform: translateY(0);
}

.action-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  transform: none;
}

.service-btn.running {
  background: linear-gradient(135deg, rgb(239, 68, 68) 0%, rgb(220, 38, 38) 100%);
  color: white;
  border-color: rgba(220, 38, 38, 0.3);
  box-shadow: 0 2px 4px rgba(239, 68, 68, 0.2), inset 0 1px 0 rgba(255, 255, 255, 0.2);
}

.service-btn.running:hover:not(:disabled) {
  background: linear-gradient(135deg, rgb(220, 38, 38) 0%, rgb(185, 28, 28) 100%);
  box-shadow: 0 4px 8px rgba(239, 68, 68, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.2);
}

.service-btn:not(.running) {
  background: linear-gradient(135deg, rgb(16, 185, 129) 0%, rgb(5, 150, 105) 100%);
  color: white;
  border-color: rgba(5, 150, 105, 0.3);
  box-shadow: 0 2px 4px rgba(16, 185, 129, 0.2), inset 0 1px 0 rgba(255, 255, 255, 0.2);
}

.service-btn:not(.running):hover:not(:disabled) {
  background: linear-gradient(135deg, rgb(5, 150, 105) 0%, rgb(4, 120, 87) 100%);
  box-shadow: 0 4px 8px rgba(16, 185, 129, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.2);
}

.restart-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, rgb(251, 191, 36) 0%, rgb(245, 158, 11) 100%);
  color: white;
  border-color: rgba(245, 158, 11, 0.3);
}

.web-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, rgb(59, 130, 246) 0%, rgb(37, 99, 235) 100%);
  color: white;
  border-color: rgba(37, 99, 235, 0.3);
}

.config-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, rgb(139, 92, 246) 0%, rgb(124, 58, 237) 100%);
  color: white;
  border-color: rgba(124, 58, 237, 0.3);
}

.test-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, rgb(6, 182, 212) 0%, rgb(8, 145, 178) 100%);
  color: white;
  border-color: rgba(8, 145, 178, 0.3);
}

.mount-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, rgb(249, 115, 22) 0%, rgb(234, 88, 12) 100%);
  color: white;
  border-color: rgba(234, 88, 12, 0.3);
}

.password-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, rgb(168, 85, 247) 0%, rgb(147, 51, 234) 100%);
  color: white;
  border-color: rgba(147, 51, 234, 0.3);
}

.service-indicator-btn {
  background: var(--color-surface);
  border-color: var(--color-border-secondary);
}

.service-indicator-btn.active {
  background: linear-gradient(135deg, rgb(239, 68, 68) 0%, rgb(220, 38, 38) 100%);
  color: white;
  border-color: rgba(5, 150, 105, 0.3);
  box-shadow: 0 2px 4px rgba(16, 185, 129, 0.2), inset 0 1px 0 rgba(255, 255, 255, 0.2);
}

.service-indicator-btn.active:hover:not(:disabled) {
  background: linear-gradient(135deg, rgb(220, 38, 38) 0%, rgb(185, 28, 28) 100%);
  border-color: rgba(220, 38, 38, 0.3);
  box-shadow: 0 4px 8px rgba(239, 68, 68, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.2);
}

.service-indicator-btn:not(.active):not(:disabled) {
  background: linear-gradient(135deg, rgb(16, 185, 129) 0%, rgb(5, 150, 105) 100%);
  color: white;
  border-color: rgba(5, 150, 105, 0.3);
  box-shadow: 0 4px 8px rgba(16, 185, 129, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.2);
}

.service-indicator-btn:not(.active):hover:not(:disabled) {
  background: linear-gradient(135deg, rgb(5, 150, 105) 0%, rgb(4, 120, 87) 100%);
  color: white;
  border-color: rgba(5, 150, 105, 0.3);
  box-shadow: 0 4px 8px rgba(16, 185, 129, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.2);
}

.settings-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, rgb(100, 116, 139) 0%, rgb(71, 85, 105) 100%);
  color: white;
  border-color: rgba(71, 85, 105, 0.3);
}

.custom-services-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, rgb(139, 92, 246) 0%, rgb(124, 58, 237) 100%);
  color: white;
  border-color: rgba(124, 58, 237, 0.3);
}

.settings-toggles {
  display: flex;
  flex-direction: row;
  gap: 1rem;
  margin-bottom: 1rem;
  padding: 0.75rem;
  background: var(--color-background-tertiary);
  border-radius: 12px;
  border: 1px solid var(--color-border-secondary);
}

.toggle-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  padding: 0.375rem;
  border-radius: 8px;
  transition: background-color 0.2s ease;
  flex: 1;
  white-space: nowrap;
}

.toggle-item:hover {
  background: var(--color-background-secondary);
}

.toggle-item input[type='checkbox'] {
  width: 1.125rem;
  height: 1.125rem;
  cursor: pointer;
  accent-color: var(--color-accent);
  border-radius: 4px;
}

.toggle-text {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  font-weight: 500;
  user-select: none;
}

@media (max-width: 768px) {
  .actions-grid {
    gap: 1.5rem;
  }

  .action-buttons {
    gap: 0.375rem;
  }

  .icon-only-btn {
    width: 1.5rem;
    padding: 0.3125rem;
  }

  .action-btn {
    padding: 0.625rem 0.75rem;
    font-size: 0.75rem;
    gap: 0.375rem;
  }

  .action-btn span {
    font-size: 0.75rem;
  }

  .section-header h4 {
    font-size: 0.9375rem;
  }

  .settings-toggles {
    gap: 0.75rem;
    padding: 0.625rem;
  }

  .toggle-text {
    font-size: 0.75rem;
  }
}

@media (max-width: 480px) {
  .action-buttons {
    flex-direction: column;
    gap: 0.375rem;
  }

  .icon-only-btn {
    width: 1.375rem;
    padding: 0.25rem;
  }

  .action-btn {
    flex: none;
    padding: 0.625rem;
    font-size: 0.75rem;
  }

  .settings-toggles {
    flex-direction: column;
    gap: 0.5rem;
  }

  .toggle-item {
    flex: none;
  }
}
</style>
