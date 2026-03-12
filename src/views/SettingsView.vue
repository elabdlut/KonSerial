<script setup lang="ts">
import { computed, onMounted } from 'vue'
import {
  NButton, NSelect, NSpace, NIcon, NSwitch, NInputNumber,
  NDivider, NScrollbar,
  useMessage
} from 'naive-ui'
import {
  ColorPaletteOutline,
  SaveOutline, RefreshOutline, InformationCircleOutline,
  ServerOutline, ShieldCheckmarkOutline
} from '@vicons/ionicons5'
import { loadConfig } from '@/stores/config'
import {
  themeSetting, fontSize, language,
  autoSave, saveInterval, maxBufferSize,
  persistSettings
} from '@/stores/settings'
import { t } from '@/stores/i18n'

const message = useMessage()

// 选项
const themeOptions = computed(() => [
  { label: t('settings.themeLight'), value: 'light' },
  { label: t('settings.themeDark'), value: 'dark' },
  { label: t('settings.themeAuto'), value: 'auto' },
])

const languageOptions = [
  { label: '简体中文', value: 'zh-CN' },
  { label: 'English', value: 'en-US' },
]

const fontSizeOptions = computed(() => [
  { label: t('settings.fontSmall'), value: 12 },
  { label: t('settings.fontMedium'), value: 14 },
  { label: t('settings.fontLarge'), value: 16 },
  { label: t('settings.fontXLarge'), value: 18 },
])

const handleSave = async () => {
  try {
    await persistSettings()
    message.success(t('settings.saved'))
  } catch (e) {
    message.error(t('settings.saveFail', String(e)))
  }
}

const handleReset = () => {
  themeSetting.value = 'light'
  language.value = 'zh-CN'
  fontSize.value = 14
  autoSave.value = true
  saveInterval.value = 60
  maxBufferSize.value = 10000
  message.info(t('settings.resetDone'))
}

onMounted(async () => {
  await loadConfig()
})
</script>

<template>
  <div class="settings-page">
    <NScrollbar>
      <div class="settings-content">
        <!-- 页面标题 -->
        <div class="page-header">
          <h1>{{ t('settings.title') }}</h1>
          <p>{{ t('settings.subtitle') }}</p>
        </div>

        <!-- 外观设置 -->
        <section class="settings-section">
          <div class="section-header">
            <NIcon :component="ColorPaletteOutline" size="20" />
            <span>{{ t('settings.appearance') }}</span>
          </div>

          <div class="settings-card">
            <div class="setting-item">
              <div class="setting-info">
                <label>{{ t('settings.theme') }}</label>
                <p>{{ t('settings.themeDesc') }}</p>
              </div>
              <NSelect
                v-model:value="themeSetting"
                :options="themeOptions"
                style="width: 150px"
              />
            </div>

            <NDivider />

            <div class="setting-item">
              <div class="setting-info">
                <label>{{ t('settings.language') }}</label>
                <p>{{ t('settings.languageDesc') }}</p>
              </div>
              <NSelect
                v-model:value="language"
                :options="languageOptions"
                style="width: 150px"
              />
            </div>

            <NDivider />

            <div class="setting-item">
              <div class="setting-info">
                <label>{{ t('settings.fontSize') }}</label>
                <p>{{ t('settings.fontSizeDesc') }}</p>
              </div>
              <NSelect
                v-model:value="fontSize"
                :options="fontSizeOptions"
                style="width: 150px"
              />
            </div>
          </div>
        </section>

        <!-- 数据设置 -->
        <section class="settings-section">
          <div class="section-header">
            <NIcon :component="ServerOutline" size="20" />
            <span>{{ t('settings.data') }}</span>
          </div>

          <div class="settings-card">
            <div class="setting-item">
              <div class="setting-info">
                <label>{{ t('settings.autoSave') }}</label>
                <p>{{ t('settings.autoSaveDesc') }}</p>
              </div>
              <NSwitch v-model:value="autoSave" />
            </div>

            <NDivider />

            <div class="setting-item">
              <div class="setting-info">
                <label>{{ t('settings.saveInterval') }}</label>
                <p>{{ t('settings.saveIntervalDesc') }}</p>
              </div>
              <NInputNumber
                v-model:value="saveInterval"
                :disabled="!autoSave"
                :min="10"
                :max="600"
                style="width: 120px"
              />
            </div>

            <NDivider />

            <div class="setting-item">
              <div class="setting-info">
                <label>{{ t('settings.bufferSize') }}</label>
                <p>{{ t('settings.bufferSizeDesc') }}</p>
              </div>
              <NInputNumber
                v-model:value="maxBufferSize"
                :min="1000"
                :max="100000"
                :step="1000"
                style="width: 120px"
              />
            </div>
          </div>
        </section>

        <!-- 关于 -->
        <section class="settings-section">
          <div class="section-header">
            <NIcon :component="InformationCircleOutline" size="20" />
            <span>{{ t('settings.about') }}</span>
          </div>

          <div class="settings-card about-card">
            <div class="app-info">
              <div class="app-logo">
                <NIcon :component="ShieldCheckmarkOutline" size="48" />
              </div>
              <div class="app-details">
                <h3>KonSerial</h3>
                <p class="version">v0.1.0</p>
                <p class="desc">{{ t('settings.appDesc') }}</p>
              </div>
            </div>
            <NDivider />
            <div class="info-grid">
              <div class="info-item">
                <span class="info-label">{{ t('settings.techStack') }}</span>
                <span class="info-value">Tauri + Vue 3 + Rust</span>
              </div>
              <div class="info-item">
                <span class="info-label">{{ t('settings.uiFramework') }}</span>
                <span class="info-value">Naive UI</span>
              </div>
              <div class="info-item">
                <span class="info-label">{{ t('settings.license') }}</span>
                <span class="info-value">MIT License</span>
              </div>
            </div>
          </div>
        </section>

        <!-- 操作按钮 -->
        <div class="actions">
          <NSpace>
            <NButton @click="handleReset">
              <template #icon><NIcon :component="RefreshOutline" /></template>
              {{ t('settings.reset') }}
            </NButton>
            <NButton type="primary" @click="handleSave">
              <template #icon><NIcon :component="SaveOutline" /></template>
              {{ t('settings.save') }}
            </NButton>
          </NSpace>
        </div>
      </div>
    </NScrollbar>
  </div>
</template>

<style scoped>
.settings-page {
  height: 100%;
  background: var(--bg-page);
}

.settings-content {
  max-width: 720px;
  margin: 0 auto;
  padding: 32px 24px;
}

.page-header {
  margin-bottom: 32px;
}

.page-header h1 {
  font-size: var(--font-3xl);
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.page-header p {
  font-size: var(--font-base);
  color: var(--text-muted);
  margin-top: 4px;
}

.settings-section {
  margin-bottom: 28px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: var(--font-lg);
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 12px;
  padding-left: 4px;
}

.settings-card {
  background: var(--bg-card);
  border-radius: 12px;
  padding: 4px 0;
  box-shadow: var(--shadow-card);
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
}

.setting-info {
  flex: 1;
}

.setting-info label {
  display: block;
  font-size: var(--font-base);
  font-weight: 500;
  color: var(--text-primary);
}

.setting-info p {
  font-size: var(--font-xs);
  color: var(--text-muted);
  margin-top: 2px;
}

.settings-card :deep(.n-divider) {
  margin: 0;
}

/* 关于卡片 */
.about-card {
  padding: 20px;
}

.app-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.app-logo {
  width: 64px;
  height: 64px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
}

.app-details h3 {
  font-size: var(--font-2xl);
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.app-details .version {
  font-size: var(--font-sm);
  color: var(--text-secondary);
  margin-top: 2px;
}

.app-details .desc {
  font-size: var(--font-sm);
  color: var(--text-muted);
  margin-top: 4px;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-top: 8px;
}

.info-item {
  text-align: center;
  padding: 12px;
  background: var(--bg-page);
  border-radius: 8px;
}

.info-label {
  display: block;
  font-size: var(--font-2xs);
  color: var(--text-muted);
  margin-bottom: 4px;
}

.info-value {
  font-size: var(--font-sm);
  font-weight: 500;
  color: var(--text-primary);
}

.actions {
  display: flex;
  justify-content: flex-end;
  padding-top: 16px;
}
</style>
