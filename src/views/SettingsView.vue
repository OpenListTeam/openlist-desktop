<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAppStore } from '../stores/app'
import { useTranslation } from '../composables/useI18n'
import { Settings, Server, HardDrive, Save, RotateCcw, AlertCircle, CheckCircle, Play } from 'lucide-vue-next'
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart'

const store = useAppStore()
const route = useRoute()
const router = useRouter()
const { t } = useTranslation()
const isSaving = ref(false)
const message = ref('')
const messageType = ref<'success' | 'error' | 'info'>('info')
const activeTab = ref('openlist')
const rcloneConfigJson = ref('')
const autoStartApp = ref(false)

const openlistCoreSettings = reactive({ ...store.settings.openlist })
const rcloneSettings = reactive({ ...store.settings.rclone })
const appSettings = reactive({ ...store.settings.app })
let originalOpenlistPort = openlistCoreSettings.port || 5244

watch(autoStartApp, async newValue => {
  if (newValue) {
    await enable()
  } else {
    await disable()
  }
})

const tabs = computed(() => [
  {
    id: 'openlist',
    label: t('settings.tabs.openlist'),
    icon: Server,
    description: t('settings.service.subtitle')
  },
  {
    id: 'rclone',
    label: t('settings.tabs.rclone'),
    icon: HardDrive,
    description: t('settings.rclone.subtitle')
  },
  {
    id: 'app',
    label: t('settings.tabs.app'),
    icon: Settings,
    description: t('settings.app.subtitle')
  }
])

onMounted(async () => {
  autoStartApp.value = await isEnabled()
  const tabParam = route.query.tab as string
  if (tabParam && ['openlist', 'rclone', 'app'].includes(tabParam)) {
    activeTab.value = tabParam
  }

  if (!openlistCoreSettings.port) openlistCoreSettings.port = 5244
  if (!openlistCoreSettings.api_token) openlistCoreSettings.api_token = ''
  if (openlistCoreSettings.auto_launch === undefined) openlistCoreSettings.auto_launch = false
  if (openlistCoreSettings.ssl_enabled === undefined) openlistCoreSettings.ssl_enabled = false

  if (!rcloneSettings.config) rcloneSettings.config = {}

  rcloneConfigJson.value = JSON.stringify(rcloneSettings.config, null, 2)
  if (!appSettings.theme) appSettings.theme = 'light'

  if (!appSettings.monitor_interval) appSettings.monitor_interval = 5
  if (appSettings.auto_update_enabled === undefined) appSettings.auto_update_enabled = true
  originalOpenlistPort = openlistCoreSettings.port || 5244
})

const hasUnsavedChanges = computed(() => {
  let rcloneConfigChanged = false
  try {
    const parsedConfig = JSON.parse(rcloneConfigJson.value)
    rcloneConfigChanged = JSON.stringify(parsedConfig) !== JSON.stringify(store.settings.rclone.config)
  } catch {
    rcloneConfigChanged = rcloneConfigJson.value !== JSON.stringify(store.settings.rclone.config, null, 2)
  }

  return (
    JSON.stringify(openlistCoreSettings) !== JSON.stringify(store.settings.openlist) ||
    JSON.stringify(rcloneSettings) !== JSON.stringify(store.settings.rclone) ||
    JSON.stringify(appSettings) !== JSON.stringify(store.settings.app) ||
    rcloneConfigChanged
  )
})

const handleSave = async () => {
  isSaving.value = true
  message.value = ''

  try {
    try {
      rcloneSettings.config = JSON.parse(rcloneConfigJson.value)
    } catch (error) {
      message.value = t('settings.rclone.config.invalidJson')
      messageType.value = 'error'
      isSaving.value = false
      return
    }

    store.settings.openlist = { ...openlistCoreSettings }
    store.settings.rclone = { ...rcloneSettings }
    store.settings.app = { ...appSettings }
    if (originalOpenlistPort !== openlistCoreSettings.port) {
      await store.saveSettingsWithUpdatePort()
    } else {
      await store.saveSettings()
    }
    message.value = t('settings.saved')
    messageType.value = 'success'
  } catch (error) {
    message.value = t('settings.saveFailed')
    messageType.value = 'error'
    console.error('Save settings error:', error)
  } finally {
    isSaving.value = false

    setTimeout(() => {
      message.value = ''
    }, 3000)
  }
}

async function startTutorial() {
  router.push({ name: 'Dashboard' })
  store.startTutorial()
}

const handleReset = async () => {
  if (!confirm(t('settings.confirmReset'))) {
    return
  }

  try {
    await store.resetSettings()
    Object.assign(openlistCoreSettings, store.settings.openlist)
    Object.assign(rcloneSettings, store.settings.rclone)
    Object.assign(appSettings, store.settings.app)

    rcloneConfigJson.value = JSON.stringify(rcloneSettings.config, null, 2)

    message.value = t('settings.resetSuccess')
    messageType.value = 'info'
  } catch (error) {
    message.value = t('settings.resetFailed')
    messageType.value = 'error'
  }
}
</script>

<template>
  <div class="settings-container">
    <div class="settings-header">
      <div class="header-content">
        <Settings :size="24" class="header-icon" />
        <div>
          <h1>{{ t('settings.title') }}</h1>
          <p>{{ t('settings.subtitle') }}</p>
        </div>
      </div>
      <div class="header-actions">
        <button @click="handleReset" class="btn btn-secondary" :title="t('settings.resetToDefaults')">
          <RotateCcw :size="16" />
          {{ t('common.reset') }}
        </button>
        <button @click="handleSave" :disabled="!hasUnsavedChanges || isSaving" class="btn btn-primary">
          <Save :size="16" />
          {{ isSaving ? t('common.saving') : t('settings.saveChanges') }}
        </button>
      </div>
    </div>

    <div v-if="message" class="message-banner" :class="messageType">
      <component :is="messageType === 'success' ? CheckCircle : AlertCircle" :size="16" />
      <span>{{ message }}</span>
      <button @click="message = ''" class="message-close">×</button>
    </div>

    <div class="tab-navigation">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        @click="activeTab = tab.id"
        class="tab-button"
        :class="{ active: activeTab === tab.id }"
      >
        <component :is="tab.icon" :size="18" />
        <span>{{ tab.label }}</span>
      </button>
    </div>

    <div class="settings-content">
      <div v-if="activeTab === 'openlist'" class="tab-content">
        <div class="settings-section">
          <h2>{{ t('settings.network.title') }}</h2>
          <p>{{ t('settings.network.subtitle') }}</p>

          <div class="form-grid">
            <div class="form-group">
              <label>{{ t('settings.service.network.port.label') }}</label>
              <input
                v-model.number="openlistCoreSettings.port"
                type="number"
                class="form-input"
                :placeholder="t('settings.service.network.port.placeholder')"
                min="1"
                max="65535"
              />
              <small>{{ t('settings.service.network.port.help') }}</small>
            </div>
            <div class="form-group">
              <label>{{ t('settings.service.network.apiToken.label') }}</label>
              <input
                v-model="openlistCoreSettings.api_token"
                type="password"
                class="form-input"
                :placeholder="t('settings.service.network.apiToken.placeholder')"
              />
              <small>{{ t('settings.service.network.apiToken.help') }}</small>
            </div>

            <div class="form-group">
              <label class="switch-label">
                <input v-model="openlistCoreSettings.ssl_enabled" type="checkbox" class="switch-input" />
                <span class="switch-slider"></span>
                <div class="switch-content">
                  <span class="switch-title">{{ t('settings.service.network.ssl.title') }}</span>
                  <span class="switch-description">{{ t('settings.service.network.ssl.description') }}</span>
                </div>
              </label>
            </div>
          </div>
        </div>

        <div class="settings-section">
          <h2>{{ t('settings.startup.title') }}</h2>
          <p>{{ t('settings.startup.subtitle') }}</p>

          <div class="form-group">
            <label class="switch-label">
              <input v-model="openlistCoreSettings.auto_launch" type="checkbox" class="switch-input" />
              <span class="switch-slider"></span>
              <div class="switch-content">
                <span class="switch-title">{{ t('settings.service.startup.autoLaunch.title') }}</span>
                <span class="switch-description">{{ t('settings.service.startup.autoLaunch.description') }}</span>
              </div>
            </label>
          </div>
        </div>
      </div>

      <div v-if="activeTab === 'rclone'" class="tab-content">
        <div class="settings-section">
          <h2>{{ t('settings.rclone.config.title') }}</h2>
          <p>{{ t('settings.rclone.config.subtitle') }}</p>

          <div class="form-group">
            <label>{{ t('settings.rclone.config.label') }}</label>
            <textarea
              v-model="rcloneConfigJson"
              class="form-textarea"
              placeholder='{ "remote1": { "type": "s3", "provider": "AWS" } }'
              rows="10"
            ></textarea>
            <small>{{ t('settings.rclone.config.tips') }}</small>
          </div>
        </div>
      </div>

      <div v-if="activeTab === 'app'" class="tab-content">
        <div class="settings-section">
          <h2>{{ t('settings.app.theme.title') }}</h2>
          <p>{{ t('settings.app.theme.subtitle') }}</p>

          <div class="form-grid">
            <div class="form-group">
              <label>{{ t('settings.theme.title') }}</label>
              <select
                v-model="appSettings.theme"
                @change="store.setTheme(appSettings.theme || 'light')"
                class="form-input"
              >
                <option value="light">{{ t('settings.app.theme.light') }}</option>
                <option value="dark">{{ t('settings.app.theme.dark') }}</option>
                <option value="auto">{{ t('settings.app.theme.auto') }}</option>
              </select>
              <small>{{ t('settings.app.theme.autoDesc') }}</small>
            </div>
          </div>
        </div>

        <div class="settings-section">
          <h2>{{ t('settings.app.monitor.title') }}</h2>
          <p>{{ t('settings.app.monitor.subtitle') }}</p>

          <div class="form-grid">
            <div class="form-group">
              <label>{{ t('settings.app.monitor.interval.label') }}</label>
              <input
                v-model.number="appSettings.monitor_interval"
                type="number"
                class="form-input"
                :placeholder="t('settings.app.monitor.interval.placeholder')"
                min="1"
                max="60"
              />
              <small>{{ t('settings.app.monitor.interval.help') }}</small>
            </div>
          </div>
        </div>

        <div class="settings-section">
          <h2>{{ t('settings.app.autoStartApp.title') }}</h2>
          <p>{{ t('settings.app.autoStartApp.subtitle') }}</p>

          <div class="form-group">
            <label class="switch-label">
              <input v-model="autoStartApp" type="checkbox" class="switch-input" />
              <span class="switch-slider"></span>
              <div class="switch-content">
                <span class="switch-title">{{ t('settings.app.autoStartApp.title') }}</span>
                <span class="switch-description">{{ t('settings.app.autoStartApp.description') }}</span>
              </div>
            </label>
          </div>
        </div>

        <div class="settings-section">
          <h2>{{ t('settings.app.updates.title') }}</h2>
          <p>{{ t('settings.app.updates.subtitle') }}</p>

          <div class="form-group">
            <label class="switch-label">
              <input v-model="appSettings.auto_update_enabled" type="checkbox" class="switch-input" />
              <span class="switch-slider"></span>
              <div class="switch-content">
                <span class="switch-title">{{ t('settings.app.updates.autoCheck.title') }}</span>
                <span class="switch-description">{{ t('settings.app.updates.autoCheck.description') }}</span>
              </div>
            </label>
          </div>
        </div>

        <div class="settings-section">
          <h2>{{ t('settings.app.tutorial.title') }}</h2>
          <p>{{ t('settings.app.tutorial.subtitle') }}</p>

          <div class="form-grid">
            <div class="form-group">
              <button @click="startTutorial" class="tutorial-btn" type="button">
                <Play :size="16" />
                {{ t('settings.app.tutorial.restart') }}
              </button>
              <small>{{ t('settings.app.tutorial.help') }}</small>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped src="./css/SettingsView.css"></style>
