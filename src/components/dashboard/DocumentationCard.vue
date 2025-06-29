<template>
  <Card :title="t('dashboard.documentation.title')" variant="elevated" hover class="documentation-card">
    <div class="doc-sections">
      <div class="docs-row">
        <div class="doc-section">
          <div class="doc-header">
            <div class="doc-info">
              <h4>{{ t('dashboard.documentation.openlist') }}</h4>
              <p class="doc-description">{{ t('dashboard.documentation.openlistDesc') }}</p>
            </div>
            <div class="doc-icon openlist-icon">
              <BookOpen :size="20" />
            </div>
          </div>
          <div class="doc-actions">
            <button @click="openOpenListDocs" class="doc-btn primary">
              <ExternalLink :size="14" />
              <span>{{ t('dashboard.documentation.openDocs') }}</span>
            </button>
            <button @click="openOpenListGitHub" class="doc-btn secondary">
              <Github :size="14" />
              <span>{{ t('dashboard.documentation.github') }}</span>
            </button>
          </div>
        </div>

        <div class="doc-section">
          <div class="doc-header">
            <div class="doc-info">
              <h4>{{ t('dashboard.documentation.rclone') }}</h4>
              <p class="doc-description">{{ t('dashboard.documentation.rcloneDesc') }}</p>
            </div>
            <div class="doc-icon rclone-icon">
              <Cloud :size="20" />
            </div>
          </div>
          <div class="doc-actions">
            <button @click="openRcloneDocs" class="doc-btn primary">
              <ExternalLink :size="14" />
              <span>{{ t('dashboard.documentation.openDocs') }}</span>
            </button>
            <button @click="openRcloneGitHub" class="doc-btn secondary">
              <Github :size="14" />
              <span>{{ t('dashboard.documentation.github') }}</span>
            </button>
          </div>
        </div>
      </div>

      <div class="quick-links">
        <div class="links-header">
          <h4>{{ t('dashboard.documentation.quickLinks') }}</h4>
        </div>
        <div class="links-grid">
          <button @click="openLink('https://docs.oplist.org/guide/api')" class="link-btn">
            <Code :size="16" />
            <span>{{ t('dashboard.documentation.apiDocs') }}</span>
          </button>
          <button @click="openLink('https://rclone.org/commands/')" class="link-btn">
            <Terminal :size="16" />
            <span>{{ t('dashboard.documentation.commands') }}</span>
          </button>
          <button @click="openLink('https://github.com/OpenListTeam/OpenList-desktop/issues')" class="link-btn">
            <HelpCircle :size="16" />
            <span>{{ t('dashboard.documentation.issues') }}</span>
          </button>
          <button @click="openLink('https://docs.oplist.org/faq/')" class="link-btn">
            <MessageCircle :size="16" />
            <span>{{ t('dashboard.documentation.faq') }}</span>
          </button>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { useTranslation } from '../../composables/useI18n'
import { ExternalLink, Github, BookOpen, Cloud, Code, Terminal, HelpCircle, MessageCircle } from 'lucide-vue-next'
import Card from '../ui/Card.vue'
import { TauriAPI } from '../../api/tauri'

const { t } = useTranslation()

const openOpenListDocs = () => {
  openLink('https://docs.oplist.org/')
}

const openOpenListGitHub = () => {
  openLink('https://github.com/OpenListTeam/OpenList')
}

const openRcloneDocs = () => {
  openLink('https://rclone.org/docs/')
}

const openRcloneGitHub = () => {
  openLink('https://github.com/rclone/rclone')
}

const openLink = async (url: string) => {
  try {
    await TauriAPI.openUrl(url)
  } catch (error) {
    console.error('Failed to open link:', error)
    window.open(url, '_blank')
  }
}
</script>

<style scoped>
.doc-sections {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.docs-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
}

.doc-section {
  border: 1px solid rgb(229 231 235);
  border-radius: 0.75rem;
  padding: 1rem;
  background: rgb(249 250 251);
  transition: all 0.2s;
}

.doc-section:hover {
  border-color: rgb(209 213 219);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

:root.dark .doc-section,
:root.auto.dark .doc-section {
  border-color: rgb(55 65 81);
  background: rgb(31 41 55);
}

:root.dark .doc-section:hover,
:root.auto.dark .doc-section:hover {
  border-color: rgb(75 85 99);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.doc-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 1rem;
}

.doc-info {
  flex: 1;
}

.doc-info h4 {
  margin: 0 0 0.5rem 0;
  font-size: 0.9375rem;
  font-weight: 600;
  color: rgb(17 24 39);
}

:root.dark .doc-info h4,
:root.auto.dark .doc-info h4 {
  color: rgb(243 244 246);
}

.doc-description {
  margin: 0;
  font-size: 0.8125rem;
  color: rgb(107 114 128);
  line-height: 1.4;
}

:root.dark .doc-description,
:root.auto.dark .doc-description {
  color: rgb(156 163 175);
}

.doc-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.5rem;
  height: 2.5rem;
  border-radius: 0.75rem;
  margin-left: 0.75rem;
  flex-shrink: 0;
}

.openlist-icon {
  background: linear-gradient(135deg, rgb(99 102 241), rgb(139 92 246));
  color: white;
}

.rclone-icon {
  background: linear-gradient(135deg, rgb(34 197 94), rgb(59 130 246));
  color: white;
}

.doc-actions {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.doc-btn {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 0.75rem;
  font-size: 0.8125rem;
  font-weight: 500;
  border: none;
  border-radius: 0.5rem;
  cursor: pointer;
  transition: all 0.2s;
  text-decoration: none;
  flex: 1;
  justify-content: center;
}

.doc-btn.primary {
  background: rgb(99 102 241);
  color: white;
}

.doc-btn.primary:hover {
  background: rgb(79 70 229);
}

.doc-btn.secondary {
  background: rgb(243 244 246);
  color: rgb(55 65 81);
  border: 1px solid rgb(209 213 219);
}

.doc-btn.secondary:hover {
  background: rgb(229 231 235);
}

:root.dark .doc-btn.secondary,
:root.auto.dark .doc-btn.secondary {
  background: rgb(55 65 81);
  color: rgb(209 213 219);
  border-color: rgb(75 85 99);
}

:root.dark .doc-btn.secondary:hover,
:root.auto.dark .doc-btn.secondary:hover {
  background: rgb(75 85 99);
}

.quick-links {
  border-top: 1px solid rgb(229 231 235);
  padding-top: 1.25rem;
}

:root.dark .quick-links,
:root.auto.dark .quick-links {
  border-top-color: rgb(55 65 81);
}

.links-header h4 {
  margin: 0 0 1rem 0;
  font-size: 0.9375rem;
  font-weight: 600;
  color: rgb(17 24 39);
}

:root.dark .links-header h4,
:root.auto.dark .links-header h4 {
  color: rgb(243 244 246);
}

.links-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(8rem, 1fr));
  gap: 0.5rem;
}

.link-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.75rem 0.5rem;
  font-size: 0.8125rem;
  font-weight: 500;
  background: white;
  color: rgb(55 65 81);
  border: 1px solid rgb(209 213 219);
  border-radius: 0.5rem;
  cursor: pointer;
  transition: all 0.2s;
  text-align: center;
}

.link-btn:hover {
  background: rgb(249 250 251);
  border-color: rgb(99 102 241);
  color: rgb(99 102 241);
}

:root.dark .link-btn,
:root.auto.dark .link-btn {
  background: rgb(17 24 39);
  color: rgb(209 213 219);
  border-color: rgb(55 65 81);
}

:root.dark .link-btn:hover,
:root.auto.dark .link-btn:hover {
  background: rgb(31 41 55);
  border-color: rgb(99 102 241);
  color: rgb(129 140 248);
}

.link-btn span {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Responsive design for smaller screens */
@media (max-width: 768px) {
  .docs-row {
    grid-template-columns: 1fr;
  }

  .doc-actions {
    flex-direction: column;
  }

  .doc-btn {
    flex: none;
  }
}
</style>
