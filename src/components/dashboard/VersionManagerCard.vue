<template>
  <Card :title="t('dashboard.versionManager.title')" variant="elevated" hover class="version-manager-card">
    <div class="version-sections">
      <div class="versions-row">
        <div class="version-item">
          <div class="version-header">
            <div class="version-title">
              <h4>{{ t('dashboard.versionManager.openlist') }}</h4>
              <span class="current-version">{{ currentVersions.openlist }}</span>
            </div>
            <button @click="refreshVersions" :disabled="refreshing" class="refresh-icon-btn">
              <component :is="refreshing ? LoaderIcon : RefreshCw" :size="16" />
            </button>
          </div>
          <div class="version-controls">
            <select v-model="selectedVersions.openlist" class="version-select" :disabled="loading.openlist">
              <option value="">{{ t('dashboard.versionManager.selectVersion') }}</option>
              <option v-for="version in availableVersions.openlist" :key="version" :value="version">
                {{ version }}
              </option>
            </select>
            <button
              @click="updateVersion('openlist')"
              :disabled="
                !selectedVersions.openlist || loading.openlist || selectedVersions.openlist === currentVersions.openlist
              "
              class="update-btn"
            >
              <component :is="loading.openlist ? LoaderIcon : Download" :size="14" />
              <span>{{ loading.openlist ? t('common.loading') : t('dashboard.versionManager.update') }}</span>
            </button>
          </div>
        </div>
        <div class="version-item">
          <div class="version-header">
            <div class="version-title">
              <h4>{{ t('dashboard.versionManager.rclone') }}</h4>
              <span class="current-version">{{ currentVersions.rclone }}</span>
            </div>
            <button @click="refreshVersions" :disabled="refreshing" class="refresh-icon-btn">
              <component :is="refreshing ? LoaderIcon : RefreshCw" :size="16" />
            </button>
          </div>
          <div class="version-controls">
            <select v-model="selectedVersions.rclone" class="version-select" :disabled="loading.rclone">
              <option value="">{{ t('dashboard.versionManager.selectVersion') }}</option>
              <option v-for="version in availableVersions.rclone" :key="version" :value="version">
                {{ version }}
              </option>
            </select>
            <button
              @click="updateVersion('rclone')"
              :disabled="
                !selectedVersions.rclone || loading.rclone || selectedVersions.rclone === currentVersions.rclone
              "
              class="update-btn"
            >
              <component :is="loading.rclone ? LoaderIcon : Download" :size="14" />
              <span>{{ loading.rclone ? t('common.loading') : t('dashboard.versionManager.update') }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useTranslation } from '../../composables/useI18n'
import { Download, RefreshCw, Loader2 as LoaderIcon } from 'lucide-vue-next'
import Card from '../ui/Card.vue'
import { TauriAPI } from '../../api/tauri'

const { t } = useTranslation()

const currentVersions = ref({
  openlist: 'unknown',
  rclone: 'unknown'
})

const selectedVersions = ref({
  openlist: '',
  rclone: ''
})

const availableVersions = ref({
  openlist: [] as string[],
  rclone: [] as string[]
})

const loading = ref({
  openlist: false,
  rclone: false
})

const refreshing = ref(false)

const getCurrentVersions = async () => {
  try {
    const [openlistVersion, rcloneVersion] = await Promise.all([
      TauriAPI.bin.version('openlist'),
      TauriAPI.bin.version('rclone')
    ])
    currentVersions.value.openlist = openlistVersion || 'unknown'
    currentVersions.value.rclone = rcloneVersion || 'unknown'
  } catch (error) {
    console.error('Failed to fetch current versions:', error)
  }
}

const fetchOpenListVersions = async () => {
  try {
    return await TauriAPI.bin.availableVersions('openlist')
  } catch (error) {
    console.error('Failed to fetch OpenList versions:', error)
    return []
  }
}

const fetchRcloneVersions = async () => {
  try {
    const versions = await TauriAPI.bin.availableVersions('rclone')
    return versions
  } catch (error) {
    return []
  }
}

const refreshVersions = async () => {
  await getCurrentVersions()
  refreshing.value = true
  try {
    const [openlistVersions, rcloneVersions] = await Promise.all([fetchOpenListVersions(), fetchRcloneVersions()])

    availableVersions.value.openlist = openlistVersions
    availableVersions.value.rclone = rcloneVersions
  } catch (error) {
    console.error('Failed to refresh versions:', error)
  } finally {
    refreshing.value = false
  }
}

const updateVersion = async (type: 'openlist' | 'rclone') => {
  loading.value[type] = true
  try {
    const result = await TauriAPI.bin.updateVersion(type, selectedVersions.value[type])

    currentVersions.value[type] = selectedVersions.value[type]
    selectedVersions.value[type] = ''

    console.log(`Updated ${type}:`, result)
  } catch (error) {
    console.error(`Failed to update ${type}:`, error)
  } finally {
    loading.value[type] = false
  }
}

onMounted(() => {
  refreshVersions()
})
</script>

<style scoped>
.version-sections {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  flex: 1;
  min-height: 0;
}

.versions-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  flex: 1;
}

.version-item {
  border: 1px solid var(--color-border-secondary, rgb(229 231 235));
  border-radius: 0.75rem;
  padding: 0.875rem;
  background: var(--color-background-tertiary, rgb(249 250 251));
  transition: all 0.2s ease;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.version-item:hover {
  border-color: var(--color-border, rgb(209 213 219));
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

:root.dark .version-item,
:root.auto.dark .version-item {
  border-color: var(--color-border-secondary, rgb(55 65 81));
  background: var(--color-background-secondary, rgb(31 41 55));
}

:root.dark .version-item:hover,
:root.auto.dark .version-item:hover {
  border-color: var(--color-border, rgb(75 85 99));
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

.version-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 0.75rem;
  gap: 0.5rem;
}

.version-title {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  flex: 1;
  min-width: 0;
}

.refresh-icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  background: var(--color-background, transparent);
  color: var(--color-text-secondary, rgb(107 114 128));
  border: 1px solid var(--color-border-secondary, rgb(209 213 219));
  border-radius: 0.5rem;
  cursor: pointer;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.refresh-icon-btn:hover:not(:disabled) {
  background: var(--color-background-secondary, rgb(229 231 235));
  color: var(--color-text-primary, rgb(17 24 39));
  border-color: var(--color-primary, rgb(99 102 241));
}

.refresh-icon-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

:root.dark .refresh-icon-btn,
:root.auto.dark .refresh-icon-btn {
  border-color: var(--color-border-secondary, rgb(55 65 81));
  color: var(--color-text-secondary, rgb(156 163 175));
}

:root.dark .refresh-icon-btn:hover:not(:disabled),
:root.auto.dark .refresh-icon-btn:hover:not(:disabled) {
  background: var(--color-background-tertiary, rgb(55 65 81));
  color: var(--color-text-primary, rgb(243 244 246));
  border-color: var(--color-primary, rgb(99 102 241));
}

.version-title h4 {
  margin: 0;
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--color-text-primary, rgb(17 24 39));
  line-height: 1.3;
}

:root.dark .version-title h4,
:root.auto.dark .version-title h4 {
  color: var(--color-text-primary, rgb(243 244 246));
}

.current-version {
  font-size: 0.75rem;
  color: var(--color-text-secondary, rgb(107 114 128));
  font-weight: 500;
  background: var(--color-background-secondary, rgb(229 231 235));
  padding: 0.25rem 0.5rem;
  border-radius: 0.375rem;
  width: fit-content;
  white-space: nowrap;
}

:root.dark .current-version,
:root.auto.dark .current-version {
  color: var(--color-text-secondary, rgb(156 163 175));
  background: var(--color-background-tertiary, rgb(55 65 81));
}

.version-controls {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  flex: 1;
}

.version-select {
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--color-border-secondary, rgb(209 213 219));
  border-radius: 0.375rem;
  text-align: center;
  background: var(--color-background, white);
  font-size: 0.875rem;
  color: var(--color-text-primary, rgb(17 24 39));
  width: 100%;
  transition: border-color 0.2s ease;
}

:root.dark .version-select,
:root.auto.dark .version-select {
  background: var(--color-background-primary, rgb(17 24 39));
  border-color: var(--color-border-secondary, rgb(55 65 81));
  color: var(--color-text-primary, rgb(243 244 246));
}

.version-select:focus {
  outline: none;
  border-color: var(--color-primary, rgb(99 102 241));
  box-shadow: 0 0 0 1px var(--color-primary, rgb(99 102 241));
}

.update-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  background: var(--color-primary, rgb(99 102 241));
  color: white;
  border: none;
  border-radius: 0.375rem;
  font-size: 0.8125rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
  width: 100%;
}

.update-btn:hover:not(:disabled) {
  background: var(--color-primary-hover, rgb(79 70 229));
}

.update-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

@media (max-width: 768px) {
  .versions-row {
    grid-template-columns: 1fr;
    gap: 0.75rem;
  }

  .version-header {
    margin-bottom: 0.5rem;
  }

  .version-item {
    padding: 0.75rem;
  }
}

.refresh-icon-btn [data-lucide='loader-2'],
.update-btn [data-lucide='loader-2'] {
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
</style>
