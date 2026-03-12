<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  NButton, NSpace, NIcon, NTooltip, NDivider, NTag,
  NScrollbar,
  useMessage
} from 'naive-ui'
import {
  DocumentOutline, FolderOpenOutline, SaveOutline, PlayOutline,
  StopOutline, AddOutline, TrashOutline, CodeSlashOutline,
  TimeOutline, CheckmarkCircleOutline, AlertCircleOutline
} from '@vicons/ionicons5'
import { t } from '@/stores/i18n'

const message = useMessage()

// 脚本状态
const scriptContent = ref(`// KonSerial 脚本示例
// 支持 JavaScript 语法

// 发送数据
async function sendHello() {
  await serial.send('Hello World!');
  console.log('已发送');
}

// 定时发送
let count = 0;
const timer = setInterval(() => {
  serial.send('Count: ' + count++);
  if (count > 10) {
    clearInterval(timer);
  }
}, 1000);
`)
const isRunning = ref(false)
const currentFile = ref('未命名脚本.js')
const isModified = ref(false)

// 运行日志
const logs = ref<{ type: string; content: string; time: string }[]>([])

// 脚本列表
const scriptFiles = ref([
  { key: '1', label: '示例脚本.js', isLeaf: true },
  { key: '2', label: '自动应答.js', isLeaf: true },
  { key: '3', label: '数据解析.js', isLeaf: true },
])

// 统计
const lineCount = computed(() => {
  return scriptContent.value.split('\n').length
})

const charCount = computed(() => {
  return scriptContent.value.length
})

// 方法
const runScript = () => {
  isRunning.value = true
  addLog('info', t('script.started'))
  message.success(t('script.startedMsg'))
}

const stopScript = () => {
  isRunning.value = false
  addLog('info', t('script.stoppedMsg'))
  message.info(t('script.stoppedMsg'))
}

const saveScript = () => {
  isModified.value = false
  message.success(t('script.savedMsg'))
}

const newScript = () => {
  scriptContent.value = '// 新脚本\n'
  currentFile.value = '未命名脚本.js'
  isModified.value = false
  message.info(t('script.newCreated'))
}

const openScript = () => {
  message.info(t('script.openWip'))
}

const addLog = (type: string, content: string) => {
  const now = new Date()
  const time = now.toLocaleTimeString('zh-CN', { hour12: false })
  logs.value.push({ type, content, time })
}

const clearLogs = () => {
  logs.value = []
}

const onContentChange = () => {
  isModified.value = true
}
</script>

<template>
  <div class="script-page">
    <!-- 左侧文件列表 -->
    <aside class="file-panel">
      <div class="panel-header">
        <div class="section-title">
          <NIcon :component="FolderOpenOutline" size="16" />
          <span>{{ t('script.files') }}</span>
        </div>
        <NTooltip>
          <template #trigger>
            <NButton size="tiny" quaternary @click="newScript">
              <template #icon><NIcon :component="AddOutline" /></template>
            </NButton>
          </template>
          {{ t('script.new') }}
        </NTooltip>
      </div>

      <NScrollbar style="flex: 1">
        <div class="file-list">
          <div 
            v-for="file in scriptFiles" 
            :key="file.key"
            class="file-item"
            :class="{ active: file.label === currentFile }"
          >
            <NIcon :component="DocumentOutline" size="14" />
            <span>{{ file.label }}</span>
          </div>
        </div>
      </NScrollbar>

      <NDivider style="margin: 12px 0" />

      <!-- 脚本信息 -->
      <div class="script-info">
        <div class="info-row">
          <span>{{ t('script.lines') }}</span>
          <span>{{ lineCount }}</span>
        </div>
        <div class="info-row">
          <span>{{ t('script.chars') }}</span>
          <span>{{ charCount }}</span>
        </div>
      </div>
    </aside>

    <!-- 中间编辑区 -->
    <main class="editor-area">
      <!-- 工具栏 -->
      <div class="editor-toolbar">
        <div class="toolbar-left">
          <NIcon :component="CodeSlashOutline" size="18" />
          <span class="file-name">{{ currentFile }}</span>
          <NTag v-if="isModified" size="small" type="warning">{{ t('script.unsaved') }}</NTag>
          <NTag v-if="isRunning" size="small" type="success">{{ t('script.running') }}</NTag>
        </div>
        <NSpace>
          <NButton size="small" @click="openScript">
            <template #icon><NIcon :component="FolderOpenOutline" /></template>
            {{ t('script.open') }}
          </NButton>
          <NButton size="small" @click="saveScript">
            <template #icon><NIcon :component="SaveOutline" /></template>
            {{ t('script.save') }}
          </NButton>
          <NDivider vertical />
          <NButton 
            :type="isRunning ? 'error' : 'primary'" 
            size="small"
            @click="isRunning ? stopScript() : runScript()"
          >
            <template #icon>
              <NIcon :component="isRunning ? StopOutline : PlayOutline" />
            </template>
            {{ isRunning ? t('script.stop') : t('script.run') }}
          </NButton>
        </NSpace>
      </div>

      <!-- 编辑器 -->
      <div class="editor-container">
        <div class="line-numbers">
          <div v-for="n in lineCount" :key="n">{{ n }}</div>
        </div>
        <textarea
          v-model="scriptContent"
          @input="onContentChange"
          class="code-editor"
          spellcheck="false"
        ></textarea>
      </div>
    </main>

    <!-- 右侧输出区 -->
    <aside class="output-panel">
      <div class="panel-header">
        <div class="section-title">
          <NIcon :component="TimeOutline" size="16" />
          <span>{{ t('script.log') }}</span>
          <NTag size="small" :bordered="false">{{ logs.length }}</NTag>
        </div>
        <NButton size="tiny" quaternary @click="clearLogs">
          <template #icon><NIcon :component="TrashOutline" /></template>
        </NButton>
      </div>

      <NScrollbar class="output-content">
        <div class="log-list">
          <div 
            v-for="(log, idx) in logs" 
            :key="idx"
            class="log-item"
            :class="log.type"
          >
            <NIcon 
              :component="log.type === 'error' ? AlertCircleOutline : CheckmarkCircleOutline" 
              size="14" 
            />
            <span class="log-time">{{ log.time }}</span>
            <span class="log-content">{{ log.content }}</span>
          </div>
          <div v-if="logs.length === 0" class="empty-log">
            {{ t('script.emptyLog') }}
          </div>
        </div>
      </NScrollbar>
    </aside>
  </div>
</template>

<style scoped>
.script-page {
  display: flex;
  height: 100%;
  background: var(--bg-page);
  gap: 16px;
  padding: 16px;
}

/* 左侧文件面板 */
.file-panel {
  width: 200px;
  flex-shrink: 0;
  background: var(--bg-card);
  border-radius: 12px;
  padding: 16px;
  box-shadow: var(--shadow-card);
  display: flex;
  flex-direction: column;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-sm);
  font-weight: 600;
  color: var(--text-primary);
}

.file-list {
  padding: 4px 0;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: var(--font-sm);
  color: var(--text-secondary);
  transition: all 0.2s;
}

.file-item:hover {
  background: var(--border-color);
}

.file-item.active {
  background: #e3f2fd;
  color: #1565c0;
}

.script-info {
  padding: 8px 0;
}

.info-row {
  display: flex;
  justify-content: space-between;
  font-size: var(--font-xs);
  color: var(--text-muted);
  padding: 4px 0;
}

/* 中间编辑区 */
.editor-area {
  flex: 1;
  background: var(--bg-card);
  border-radius: 12px;
  box-shadow: var(--shadow-card);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

.editor-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-page);
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-name {
  font-size: var(--font-base);
  font-weight: 500;
  color: var(--text-primary);
}

.editor-container {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.line-numbers {
  width: 48px;
  background: var(--bg-page);
  padding: 16px 8px;
  text-align: right;
  font-family: 'SF Mono', Monaco, Consolas, monospace;
  font-size: var(--font-sm);
  line-height: 1.6;
  color: var(--text-muted);
  border-right: 1px solid var(--border-color);
  overflow: hidden;
  user-select: none;
}

.code-editor {
  flex: 1;
  padding: 16px;
  border: none;
  outline: none;
  resize: none;
  font-family: 'SF Mono', Monaco, Consolas, monospace;
  font-size: var(--font-sm);
  line-height: 1.6;
  color: var(--text-primary);
  background: var(--bg-card);
}

/* 右侧输出面板 */
.output-panel {
  width: 280px;
  flex-shrink: 0;
  background: var(--bg-card);
  border-radius: 12px;
  box-shadow: var(--shadow-card);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.output-panel .panel-header {
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
  margin-bottom: 0;
}

.output-content {
  flex: 1;
}

.log-list {
  padding: 12px;
}

.log-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 8px;
  border-radius: 6px;
  font-size: var(--font-xs);
  margin-bottom: 4px;
}

.log-item.info {
  background: var(--bg-page);
  color: var(--text-secondary);
}

.log-item.error {
  background: #ffebee;
  color: #c62828;
}

.log-item.success {
  background: #e8f5e9;
  color: #2e7d32;
}

.log-time {
  color: var(--text-muted);
  flex-shrink: 0;
}

.log-content {
  flex: 1;
}

.empty-log {
  text-align: center;
  color: var(--text-muted);
  padding: 32px;
  font-size: var(--font-sm);
}
</style>
