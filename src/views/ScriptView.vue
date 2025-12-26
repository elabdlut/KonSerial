<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  NButton, NSelect, NSpace, NIcon, NInput, NTooltip, NDivider, NTag,
  NScrollbar, NTree,
  useMessage
} from 'naive-ui'
import {
  DocumentOutline, FolderOpenOutline, SaveOutline, PlayOutline,
  StopOutline, AddOutline, TrashOutline, CodeSlashOutline,
  TimeOutline, CheckmarkCircleOutline, AlertCircleOutline
} from '@vicons/ionicons5'

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
  addLog('info', '脚本开始执行')
  message.success('脚本已启动')
}

const stopScript = () => {
  isRunning.value = false
  addLog('info', '脚本已停止')
  message.info('脚本已停止')
}

const saveScript = () => {
  isModified.value = false
  message.success('脚本已保存')
}

const newScript = () => {
  scriptContent.value = '// 新脚本\n'
  currentFile.value = '未命名脚本.js'
  isModified.value = false
  message.info('已创建新脚本')
}

const openScript = () => {
  message.info('打开文件功能开发中...')
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
          <span>脚本文件</span>
        </div>
        <NTooltip>
          <template #trigger>
            <NButton size="tiny" quaternary @click="newScript">
              <template #icon><NIcon :component="AddOutline" /></template>
            </NButton>
          </template>
          新建脚本
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
          <span>行数</span>
          <span>{{ lineCount }}</span>
        </div>
        <div class="info-row">
          <span>字符</span>
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
          <NTag v-if="isModified" size="small" type="warning">未保存</NTag>
          <NTag v-if="isRunning" size="small" type="success">运行中</NTag>
        </div>
        <NSpace>
          <NButton size="small" @click="openScript">
            <template #icon><NIcon :component="FolderOpenOutline" /></template>
            打开
          </NButton>
          <NButton size="small" @click="saveScript">
            <template #icon><NIcon :component="SaveOutline" /></template>
            保存
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
            {{ isRunning ? '停止' : '运行' }}
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
          <span>运行日志</span>
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
            运行脚本后将在此显示日志
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
  background: #f5f7fa;
  gap: 16px;
  padding: 16px;
}

/* 左侧文件面板 */
.file-panel {
  width: 200px;
  flex-shrink: 0;
  background: #fff;
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.08);
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
  font-size: 13px;
  font-weight: 600;
  color: #333;
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
  font-size: 13px;
  color: #555;
  transition: all 0.2s;
}

.file-item:hover {
  background: #f5f5f5;
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
  font-size: 12px;
  color: #888;
  padding: 4px 0;
}

/* 中间编辑区 */
.editor-area {
  flex: 1;
  background: #fff;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.08);
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
  border-bottom: 1px solid #eee;
  background: #fafafa;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-name {
  font-size: 14px;
  font-weight: 500;
  color: #333;
}

.editor-container {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.line-numbers {
  width: 48px;
  background: #f8f9fa;
  padding: 16px 8px;
  text-align: right;
  font-family: 'SF Mono', Monaco, Consolas, monospace;
  font-size: 13px;
  line-height: 1.6;
  color: #999;
  border-right: 1px solid #eee;
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
  font-size: 13px;
  line-height: 1.6;
  color: #333;
  background: #fff;
}

/* 右侧输出面板 */
.output-panel {
  width: 280px;
  flex-shrink: 0;
  background: #fff;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.08);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.output-panel .panel-header {
  padding: 12px 16px;
  border-bottom: 1px solid #eee;
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
  font-size: 12px;
  margin-bottom: 4px;
}

.log-item.info {
  background: #f5f5f5;
  color: #666;
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
  color: #999;
  flex-shrink: 0;
}

.log-content {
  flex: 1;
}

.empty-log {
  text-align: center;
  color: #999;
  padding: 32px;
  font-size: 13px;
}
</style>
