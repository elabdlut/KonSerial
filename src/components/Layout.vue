<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, RouterView, useRoute } from 'vue-router'
import { isDark } from '@/stores/settings'
import { t } from '@/stores/i18n'

const route = useRoute()

const menuItems = computed(() => [
  { path: '/serial', icon: '🔌', label: t('nav.serial') },
  { path: '/chart', icon: '📊', label: t('nav.chart') },
  { path: '/script', icon: '📝', label: t('nav.script') },
  { path: '/history', icon: '📜', label: t('nav.history') },
  { path: '/settings', icon: '⚙️', label: t('nav.settings') }
])
</script>

<template>
  <div class="layout" :class="{ dark: isDark }">
    <aside class="sidebar">
      <div class="sidebar-header">
        <h1>KonSerial</h1>
        <p>{{ t('app.subtitle') }}</p>
      </div>
      
      <nav class="sidebar-nav">
        <RouterLink
          v-for="item in menuItems"
          :key="item.path"
          :to="item.path"
          class="nav-item"
          :class="{ active: route.path === item.path }"
        >
          <span class="nav-icon">{{ item.icon }}</span>
          <span class="nav-label">{{ item.label }}</span>
        </RouterLink>
      </nav>
    </aside>

    <main class="main-content">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.layout {
  display: flex;
  height: 100vh;
  background: var(--bg-page);
  transition: background 0.3s;
}

.sidebar {
  width: 220px;
  flex-shrink: 0;
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  transition: background 0.3s, border-color 0.3s;
}

.sidebar-header {
  padding: 24px 20px 16px;
  border-bottom: 1px solid var(--border-color);
}

.sidebar-header h1 {
  font-size: var(--font-2xl);
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
}

.sidebar-header p {
  font-size: var(--font-xs);
  color: var(--text-muted);
  margin-top: 4px;
}

.sidebar-nav {
  padding: 12px;
  flex: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  padding: 10px 14px;
  margin-bottom: 4px;
  border-radius: 8px;
  text-decoration: none;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.nav-item:hover {
  background: var(--border-color);
}

.nav-item.active {
  background: #4098fc;
  color: #fff;
}

.nav-icon {
  font-size: var(--font-xl);
  margin-right: 10px;
}

.nav-label {
  font-size: var(--font-base);
  font-weight: 500;
}

.main-content {
  flex: 1;
  overflow: auto;
}
</style>
