<script setup lang="ts">
import { ref, onMounted, provide, watch } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useThemeStore } from './stores/theme'
import { useLocaleStore, type Locale } from './stores/locale'
import { i18n } from './i18n'

const router = useRouter()
const { t } = useI18n()
const themeStore = useThemeStore()
const localeStore = useLocaleStore()
const isLoading = ref(true)
const freshnessMessage = ref('')
const shouldRedirectToScraper = ref(false)

provide('freshnessMessage', freshnessMessage.value)

const localeOptions: { value: Locale; label: string }[] = [
  { value: 'ja', label: '日本語' },
  { value: 'en', label: 'English' },
  { value: 'zh-TW', label: '中文' },
]

watch(() => localeStore.locale, (newLocale) => {
  i18n.global.locale.value = newLocale
})

onMounted(async () => {
  try {
    const freshness = await invoke<{ is_fresh: boolean; message: string }>('check_database_freshness')
    
    if (!freshness.is_fresh) {
      freshnessMessage.value = freshness.message
      shouldRedirectToScraper.value = true
      router.push('/scraper')
    }
  } catch (e) {
    console.error('Failed to check freshness:', e)
  } finally {
    isLoading.value = false
  }
})
</script>

<template>
  <div class="app-container" :data-theme="themeStore.theme">
    <nav class="sidebar">
      <div class="logo">
        <h2>VOCALOID Search</h2>
      </div>
      <router-link to="/" class="nav-item">
        <span class="icon">🔍</span>
        <span>{{ t('app.search') }}</span>
      </router-link>
      <router-link to="/history" class="nav-item">
        <span class="icon">📜</span>
        <span>{{ t('app.history') }}</span>
      </router-link>
      <router-link to="/scraper" class="nav-item">
        <span class="icon">🔄</span>
        <span>{{ t('app.scraper') }}</span>
      </router-link>
      <div class="nav-footer">
        <div class="settings-row">
          <button class="theme-toggle" @click="themeStore.toggle()" :title="themeStore.theme === 'dark' ? t('app.lightMode') : t('app.darkMode')">
            <span v-if="themeStore.theme === 'dark'">☀️</span>
            <span v-else>🌙</span>
          </button>
          <select class="locale-select" v-model="localeStore.locale">
            <option v-for="opt in localeOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
          </select>
        </div>
      </div>
    </nav>
    
    <main class="main-content">
      <div v-if="isLoading" class="loading">
        <div class="spinner"></div>
        <p>載入中...</p>
      </div>
      <router-view v-else />
    </main>
  </div>
</template>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
  background: var(--bg-primary);
}

.sidebar {
  width: 200px;
  background: var(--bg-surface);
  border-right: 1px solid var(--border-color);
  padding: 1rem;
  display: flex;
  flex-direction: column;
}

.logo h2 {
  font-size: 1.1rem;
  color: var(--text-primary);
  margin-bottom: 2rem;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  border-radius: 8px;
  color: var(--text-secondary);
  text-decoration: none;
  transition: all 0.2s;
  margin-bottom: 0.25rem;
}

.nav-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.nav-item.router-link-active {
  background: var(--accent-primary);
  color: white;
}

.icon {
  font-size: 1.2rem;
}

.main-content {
  flex: 1;
  overflow: hidden;
}

.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-muted);
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--border-color);
  border-top-color: var(--accent-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.nav-footer {
  margin-top: auto;
  padding-top: 1rem;
  border-top: 1px solid var(--border-color);
}

.settings-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

.theme-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 8px;
  background: var(--bg-hover);
  border: 1px solid var(--border-color);
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 18px;
}

.theme-toggle:hover {
  background: var(--accent-primary);
  border-color: var(--accent-primary);
}

.locale-select {
  padding: 6px 10px;
  border-radius: 8px;
  background: var(--bg-hover);
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.locale-select:hover {
  border-color: var(--accent-primary);
}

.locale-select:focus {
  outline: none;
  border-color: var(--accent-primary);
}
</style>
