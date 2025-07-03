<script setup lang="ts">
import { computed } from 'vue'
import { useTranslation } from '../composables/useI18n'
import { useAppStore } from '../stores/app'
import LanguageSwitcher from './ui/LanguageSwitcher.vue'
import ThemeSwitcher from './ui/ThemeSwitcher.vue'

import { Home, HardDrive, FileText, Settings, Download, DownloadCloud, Github } from 'lucide-vue-next'
import { TauriAPI } from '@/api/tauri'

const { t } = useTranslation()
const appStore = useAppStore()

const navigationItems = computed(() => [
  { name: t('navigation.dashboard'), path: '/', icon: Home, shortcut: 'Ctrl+H' },
  { name: t('navigation.mount'), path: '/mount', icon: HardDrive, shortcut: 'Ctrl+M' },
  { name: t('navigation.logs'), path: '/logs', icon: FileText, shortcut: 'Ctrl+L' },
  { name: t('navigation.settings'), path: '/settings', icon: Settings, shortcut: 'Ctrl+,' },
  {
    name: t('navigation.update'),
    path: '/update',
    icon: appStore.updateAvailable ? DownloadCloud : Download,
    shortcut: 'Ctrl+U',
    hasNotification: appStore.updateAvailable
  }
])

const openLink = async (url: string) => {
  try {
    await TauriAPI.files.url(url)
  } catch (error) {
    console.error('Failed to open link:', error)
    window.open(url, '_blank')
  }
}
</script>

<template>
  <nav class="navigation">
    <div class="title-bar">
      <div class="app-title">
        <div class="app-icon">
          <HardDrive :size="20" />
        </div>
        <span class="title-text">{{ t('app.title') }}</span>
      </div>
    </div>

    <div class="language-section">
      <LanguageSwitcher />
    </div>

    <div class="theme-section">
      <ThemeSwitcher />
    </div>

    <div class="nav-menu">
      <router-link
        v-for="item in navigationItems"
        :key="item.path"
        :to="item.path"
        class="nav-item"
        :class="{ 'has-notification': item.hasNotification }"
        :title="`${item.name} (${item.shortcut})`"
      >
        <div class="nav-icon-container">
          <component :is="item.icon" :size="18" />
          <div v-if="item.hasNotification" class="notification-dot"></div>
        </div>
        <span>{{ item.name }}</span>
      </router-link>
    </div>

    <div class="github-section">
      <a
        @click.prevent="openLink('https://github.com/OpenListTeam/openlist-desktop')"
        class="github-link"
        title="View on GitHub"
      >
        <Github :size="20" />
      </a>
    </div>
  </nav>
</template>

<style scoped>
.navigation {
  display: flex;
  flex-direction: column;
  width: 150px;
  height: 100vh;
  background: var(--color-background-secondary);
  border-right: 1px solid rgb(229 231 235);
  overflow: hidden;
}

:root.dark .navigation,
:root.auto.dark .navigation {
  background: var(--color-background-secondary);
  border-right-color: var(--color-background-secondary);
}

.title-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem;
  border-bottom: 1px solid var(--color-background-secondary);
}

:root.dark .title-bar,
:root.auto.dark .title-bar {
  border-bottom-color: var(--color-background-secondary);
}

.app-title {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.app-icon {
  display: flex;
  align-items: center;
  color: rgb(133, 135, 242);
}

.title-text {
  font-size: 1.125rem;
  font-weight: 600;
  color: rgb(17 24 39);
}

:root.dark .title-text,
:root.auto.dark .title-text {
  color: rgb(243 244 246);
}

.service-section {
  padding: 1rem;
  border-bottom: 1px solid rgb(229 231 235);
}

:root.dark .service-section,
:root.auto.dark .service-section {
  border-bottom-color: rgb(55 65 81);
}

.language-section {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.75rem;
  border-bottom: 1px solid rgb(229 231 235);
}

:root.dark .language-section,
:root.auto.dark .language-section {
  border-bottom-color: rgb(55 65 81);
}

.theme-section {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.75rem;
  border-bottom: 1px solid rgb(229 231 235);
}

:root.dark .theme-section,
:root.auto.dark .theme-section {
  border-bottom-color: rgb(55 65 81);
}

.nav-menu {
  flex: 1;
  padding: 1rem 0;
  overflow-y: auto;
  min-height: 0;
}

.nav-item {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  color: rgb(75 85 99);
  text-decoration: none;
  font-size: 0.875rem;
  font-weight: 500;
  transition: all 0.2s ease-in-out;
}

:root.dark .nav-item,
:root.auto.dark .nav-item {
  color: rgb(209 213 219);
}

.nav-item:hover {
  background: rgb(243 244 246);
  color: rgb(17 24 39);
}

:root.dark .nav-item:hover,
:root.auto.dark .nav-item:hover {
  background: rgb(55 65 81);
  color: rgb(243 244 246);
}

.nav-item.router-link-active {
  background: rgb(239 246 255);
  color: rgb(99 102 241);
  border-right: 3px solid rgb(99 102 241);
}

:root.dark .nav-item.router-link-active,
:root.auto.dark .nav-item.router-link-active {
  background: rgb(30 58 138 / 0.2);
  color: rgb(129 140 248);
  border-right-color: rgb(129 140 248);
}

.nav-icon-container {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.notification-dot {
  position: absolute;
  top: -4px;
  right: -4px;
  width: 8px;
  height: 8px;
  background: rgb(220 38 38);
  border-radius: 50%;
  border: 2px solid var(--color-background-secondary);
  animation: pulse 2s infinite;
}

.nav-item.has-notification .nav-icon-container {
  color: rgb(220 38 38);
}

:root.dark .nav-item.has-notification .nav-icon-container,
:root.auto.dark .nav-item.has-notification .nav-icon-container {
  color: rgb(248 113 113);
}

.github-section {
  position: absolute;
  bottom: 0;
  padding: 0.5rem;
  display: flex;
  justify-content: center;
  align-items: center;
  margin-top: auto;
}

:root.dark .github-section,
:root.auto.dark .github-section {
  border-top-color: rgb(55 65 81);
}

.github-link {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.5rem;
  color: rgb(75 85 99);
  text-decoration: none;
  border-radius: 0.5rem;
  transition: all 0.2s ease-in-out;
}

:root.dark .github-link,
:root.auto.dark .github-link {
  color: rgb(209 213 219);
}

.github-link:hover {
  background: rgb(243 244 246);
  color: rgb(17 24 39);
  transform: scale(1.1);
}

:root.dark .github-link:hover,
:root.auto.dark .github-link:hover {
  background: rgb(55 65 81);
  color: rgb(243 244 246);
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}
</style>
