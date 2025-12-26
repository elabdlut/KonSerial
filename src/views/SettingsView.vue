<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  NButton, NSelect, NSpace, NIcon, NSwitch, NInputNumber,
  NDivider, NTag, NScrollbar,
  useMessage
} from 'naive-ui'
import {
  SettingsOutline, ColorPaletteOutline, LanguageOutline,
  SaveOutline, RefreshOutline, InformationCircleOutline,
  FolderOutline, ServerOutline, ShieldCheckmarkOutline
} from '@vicons/ionicons5'
import { appConfig, loadConfig, saveConfig } from '@/stores/config'

const message = useMessage()

// 设置项
const theme = ref('light')
const language = ref('zh-CN')
const autoSave = ref(true)
const saveInterval = ref(60)
const maxBufferSize = ref(10000)
const fontSize = ref(14)

// 主题选项
const themeOptions = [
  { label: '浅色主题', value: 'light' },
  { label: '深色主题', value: 'dark' },
  { label: '跟随系统', value: 'auto' },
]

const languageOptions = [
  { label: '简体中文', value: 'zh-CN' },
  { label: 'English', value: 'en-US' },
]

const fontSizeOptions = [
  { label: '小 (12px)', value: 12 },
  { label: '中 (14px)', value: 14 },
  { label: '大 (16px)', value: 16 },
  { label: '特大 (18px)', value: 18 },
]

const handleSave = async () => {
  try {
    if (appConfig.value) {
      appConfig.value.ui.theme = theme.value
      appConfig.value.ui.language = language.value
      appConfig.value.ui.font_size = fontSize.value
      appConfig.value.data.auto_save = autoSave.value
      appConfig.value.data.save_interval = saveInterval.value
      appConfig.value.data.max_buffer_size = maxBufferSize.value
      await saveConfig()
    }
    message.success('设置已保存')
  } catch (e) {
    message.error(`保存失败: ${e}`)
  }
}

const handleReset = () => {
  theme.value = 'light'
  language.value = 'zh-CN'
  autoSave.value = true
  saveInterval.value = 60
  maxBufferSize.value = 10000
  fontSize.value = 14
  message.info('已恢复默认设置')
}

onMounted(async () => {
  await loadConfig()
  if (appConfig.value) {
    theme.value = appConfig.value.ui.theme
    language.value = appConfig.value.ui.language
    fontSize.value = appConfig.value.ui.font_size
    autoSave.value = appConfig.value.data.auto_save
    saveInterval.value = appConfig.value.data.save_interval
    maxBufferSize.value = appConfig.value.data.max_buffer_size
  }
})
</script>

<template>
  <div class="settings-page">
    <NScrollbar>
      <div class="settings-content">
        <!-- 页面标题 -->
        <div class="page-header">
          <h1>设置</h1>
          <p>应用配置与偏好设置</p>
        </div>

        <!-- 外观设置 -->
        <section class="settings-section">
          <div class="section-header">
            <NIcon :component="ColorPaletteOutline" size="20" />
            <span>外观</span>
          </div>

          <div class="settings-card">
            <div class="setting-item">
              <div class="setting-info">
                <label>主题</label>
                <p>选择应用的显示主题</p>
              </div>
              <NSelect
                v-model:value="theme"
                :options="themeOptions"
                style="width: 150px"
              />
            </div>

            <NDivider />

            <div class="setting-item">
              <div class="setting-info">
                <label>语言</label>
                <p>选择界面显示语言</p>
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
                <label>字体大小</label>
                <p>调整界面字体大小</p>
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
            <span>数据</span>
          </div>

          <div class="settings-card">
            <div class="setting-item">
              <div class="setting-info">
                <label>自动保存</label>
                <p>定时自动保存接收的数据</p>
              </div>
              <NSwitch v-model:value="autoSave" />
            </div>

            <NDivider />

            <div class="setting-item">
              <div class="setting-info">
                <label>保存间隔</label>
                <p>自动保存的时间间隔（秒）</p>
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
                <label>缓冲区大小</label>
                <p>最大数据缓冲条数</p>
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
            <span>关于</span>
          </div>

          <div class="settings-card about-card">
            <div class="app-info">
              <div class="app-logo">
                <NIcon :component="ShieldCheckmarkOutline" size="48" />
              </div>
              <div class="app-details">
                <h3>KonSerial</h3>
                <p class="version">v0.1.0</p>
                <p class="desc">现代化轻量级串口调试工具</p>
              </div>
            </div>
            <NDivider />
            <div class="info-grid">
              <div class="info-item">
                <span class="info-label">技术栈</span>
                <span class="info-value">Tauri + Vue 3 + Rust</span>
              </div>
              <div class="info-item">
                <span class="info-label">UI 框架</span>
                <span class="info-value">Naive UI</span>
              </div>
              <div class="info-item">
                <span class="info-label">许可证</span>
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
              恢复默认
            </NButton>
            <NButton type="primary" @click="handleSave">
              <template #icon><NIcon :component="SaveOutline" /></template>
              保存设置
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
  background: #f5f7fa;
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
  font-size: 28px;
  font-weight: 600;
  color: #1a1a1a;
  margin: 0;
}

.page-header p {
  font-size: 14px;
  color: #888;
  margin-top: 4px;
}

.settings-section {
  margin-bottom: 28px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 600;
  color: #333;
  margin-bottom: 12px;
  padding-left: 4px;
}

.settings-card {
  background: #fff;
  border-radius: 12px;
  padding: 4px 0;
  box-shadow: 0 1px 3px rgba(0,0,0,0.08);
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
  font-size: 14px;
  font-weight: 500;
  color: #333;
}

.setting-info p {
  font-size: 12px;
  color: #888;
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
  font-size: 20px;
  font-weight: 600;
  color: #333;
  margin: 0;
}

.app-details .version {
  font-size: 13px;
  color: #666;
  margin-top: 2px;
}

.app-details .desc {
  font-size: 13px;
  color: #999;
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
  background: #f8f9fa;
  border-radius: 8px;
}

.info-label {
  display: block;
  font-size: 11px;
  color: #999;
  margin-bottom: 4px;
}

.info-value {
  font-size: 13px;
  font-weight: 500;
  color: #333;
}

.actions {
  display: flex;
  justify-content: flex-end;
  padding-top: 16px;
}
</style>
