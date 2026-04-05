<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import {
  NButton, NSpace, NIcon, NTooltip, NDivider, NTag,
  NScrollbar, NSelect,
  useMessage
} from 'naive-ui'
import {
  DocumentOutline, FolderOpenOutline, SaveOutline, PlayOutline,
  StopOutline, AddOutline, TrashOutline, CodeSlashOutline,
  TimeOutline, CheckmarkCircleOutline, AlertCircleOutline,
  CloseOutline
} from '@vicons/ionicons5'
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog'
import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs'
import { sendData, activeConnections } from '@/stores/serial'
import { t } from '@/stores/i18n'

const message = useMessage()

// 目标连接选择
const targetConnectionId = ref<string | null>(null)

const connectionOptions = computed(() =>
  activeConnections.value.map(c => ({
    label: `${c.config.port_name} (${c.status === 'Connected' ? '已连接' : '未连接'})`,
    value: c.connection_id,
  }))
)

// ========== 脚本文件管理 ==========

interface ScriptFile {
  id: string
  name: string
  content: string
  path: string | null  // null = 未保存到磁盘
  modified: boolean
}

const scripts = ref<ScriptFile[]>([
  {
    id: 'default',
    name: 'example.js',
    path: null,
    modified: false,
    content: `// KonSerial Script Example
// Available API:
//   serial.send(text)    - Send text to serial port
//   serial.sendHex(hex)  - Send hex data (e.g. "01 02 FF")
//   sleep(ms)            - Wait for specified milliseconds
//   console.log(...)     - Print to log panel

// Example: Send hello message
await serial.send("Hello from KonSerial!\\n");
console.log("Message sent!");

// Example: Periodic data sending
for (let i = 0; i < 5; i++) {
  await serial.send("count:" + i + "\\n");
  await sleep(1000);
}
console.log("Done!");
`,
  },
])

const activeScriptId = ref('default')
const isRunning = ref(false)

// 当前活跃脚本
const activeScript = computed(() =>
  scripts.value.find(s => s.id === activeScriptId.value)
)

const scriptContent = computed({
  get: () => activeScript.value?.content ?? '',
  set: (val: string) => {
    const s = scripts.value.find(s => s.id === activeScriptId.value)
    if (s) {
      s.content = val
      s.modified = true
    }
  },
})

const lineCount = computed(() => scriptContent.value.split('\n').length)
const charCount = computed(() => scriptContent.value.length)

// ========== 运行日志 ==========

interface LogEntry {
  type: 'info' | 'error' | 'success' | 'warn'
  content: string
  time: string
}
const logs = ref<LogEntry[]>([])
const logScrollRef = ref()

const addLog = (type: LogEntry['type'], content: string) => {
  const now = new Date()
  const time = now.toLocaleTimeString('zh-CN', { hour12: false })
  logs.value.push({ type, content, time })
  // 限制日志数量
  if (logs.value.length > 500) logs.value.splice(0, logs.value.length - 500)
}
const clearLogs = () => { logs.value = [] }

// ========== 脚本执行引擎 ==========

let runningAbort: AbortController | null = null
let activeTimers: number[] = []

const runScript = async () => {
  if (isRunning.value) return
  if (!targetConnectionId.value) {
    message.warning(t('script.noConnection'))
    addLog('warn', t('script.noConnection'))
    return
  }

  isRunning.value = true
  addLog('info', t('script.started'))
  message.success(t('script.startedMsg'))
  runningAbort = new AbortController()

  const code = scriptContent.value

  // 构建沙盒 API
  const serialApi = {
    send: async (data: string) => {
      if (runningAbort?.signal.aborted) throw new Error('Script stopped')
      try {
        await sendData(targetConnectionId.value!, data, false)
        addLog('info', `TX: ${data.replace(/\n/g, '\\n')}`)
      } catch (e) {
        addLog('error', `Send failed: ${String(e)}`)
        throw e
      }
    },
    sendHex: async (hex: string) => {
      if (runningAbort?.signal.aborted) throw new Error('Script stopped')
      try {
        await sendData(targetConnectionId.value!, hex, true)
        addLog('info', `TX(HEX): ${hex}`)
      } catch (e) {
        addLog('error', `Send failed: ${String(e)}`)
        throw e
      }
    },
  }

  const sleepFn = (ms: number) => {
    return new Promise<void>((resolve, reject) => {
      if (runningAbort?.signal.aborted) { reject(new Error('Script stopped')); return }
      const id = window.setTimeout(() => {
        activeTimers = activeTimers.filter(t => t !== id)
        if (runningAbort?.signal.aborted) { reject(new Error('Script stopped')); return }
        resolve()
      }, ms)
      activeTimers.push(id)
      runningAbort?.signal.addEventListener('abort', () => {
        clearTimeout(id)
        reject(new Error('Script stopped'))
      })
    })
  }

  // Console 拦截
  const consoleMock = {
    log: (...args: unknown[]) => addLog('info', args.map(String).join(' ')),
    warn: (...args: unknown[]) => addLog('warn', args.map(String).join(' ')),
    error: (...args: unknown[]) => addLog('error', args.map(String).join(' ')),
    info: (...args: unknown[]) => addLog('info', args.map(String).join(' ')),
  }

  try {
    // 用 AsyncFunction 构建沙盒
    const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor
    const fn = new AsyncFunction('serial', 'sleep', 'console', code)
    await fn(serialApi, sleepFn, consoleMock)
    if (!runningAbort?.signal.aborted) {
      addLog('success', t('script.completed'))
    }
  } catch (e: unknown) {
    const msg = String(e)
    if (!msg.includes('Script stopped')) {
      addLog('error', `Error: ${msg}`)
    }
  } finally {
    cleanupTimers()
    isRunning.value = false
    runningAbort = null
  }
}

const stopScript = () => {
  if (!isRunning.value) return
  runningAbort?.abort()
  cleanupTimers()
  isRunning.value = false
  addLog('info', t('script.stoppedMsg'))
  message.info(t('script.stoppedMsg'))
}

const cleanupTimers = () => {
  activeTimers.forEach(id => clearTimeout(id))
  activeTimers = []
}

// ========== 文件操作 ==========

let nextId = 1

const newScript = () => {
  const id = `script_${Date.now()}_${nextId++}`
  scripts.value.push({
    id,
    name: `untitled_${nextId}.js`,
    path: null,
    modified: false,
    content: '// New script\n\nawait serial.send("Hello!\\n");\n',
  })
  activeScriptId.value = id
  message.info(t('script.newCreated'))
}

const selectScript = (id: string) => {
  activeScriptId.value = id
}

const removeScript = (id: string) => {
  if (scripts.value.length <= 1) return
  const idx = scripts.value.findIndex(s => s.id === id)
  if (idx < 0) return
  scripts.value.splice(idx, 1)
  if (activeScriptId.value === id) {
    activeScriptId.value = scripts.value[0].id
  }
}

const openFile = async () => {
  try {
    const selected = await openDialog({
      multiple: false,
      filters: [{ name: 'Script', extensions: ['js', 'ts', 'txt'] }],
    })
    if (!selected) return

    const filePath = selected as string
    const content = await readTextFile(filePath)
    const name = filePath.split(/[\/\\]/).pop() || 'unknown.js'

    // 检查是否已打开
    const existing = scripts.value.find(s => s.path === filePath)
    if (existing) {
      existing.content = content
      existing.modified = false
      activeScriptId.value = existing.id
      return
    }

    const id = `file_${Date.now()}_${nextId++}`
    scripts.value.push({ id, name, path: filePath, content, modified: false })
    activeScriptId.value = id
    message.success(t('script.openedMsg'))
  } catch (e) {
    message.error(`Open failed: ${String(e)}`)
  }
}

const saveFile = async () => {
  const s = activeScript.value
  if (!s) return

  try {
    if (s.path) {
      await writeTextFile(s.path, s.content)
    } else {
      const filePath = await saveDialog({
        filters: [{ name: 'Script', extensions: ['js'] }],
        defaultPath: s.name,
      })
      if (!filePath) return
      await writeTextFile(filePath, s.content)
      s.path = filePath
      s.name = filePath.split(/[\/\\]/).pop() || s.name
    }
    s.modified = false
    message.success(t('script.savedMsg'))
  } catch (e) {
    message.error(`Save failed: ${String(e)}`)
  }
}

const onContentChange = () => {
  const s = scripts.value.find(s => s.id === activeScriptId.value)
  if (s) s.modified = true
}

onUnmounted(() => {
  stopScript()
})
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
            v-for="file in scripts"
            :key="file.id"
            class="file-item"
            :class="{ active: file.id === activeScriptId }"
            @click="selectScript(file.id)"
          >
            <NIcon :component="DocumentOutline" size="14" />
            <span class="file-label">{{ file.name }}</span>
            <span v-if="file.modified" class="modified-dot">●</span>
            <NButton
              v-if="scripts.length > 1"
              size="tiny"
              quaternary
              class="close-btn"
              @click.stop="removeScript(file.id)"
            >
              <template #icon><NIcon :component="CloseOutline" size="12" /></template>
            </NButton>
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
          <span class="file-name">{{ activeScript?.name }}</span>
          <NTag v-if="activeScript?.modified" size="small" type="warning">{{ t('script.unsaved') }}</NTag>
          <NTag v-if="isRunning" size="small" type="success">{{ t('script.running') }}</NTag>
        </div>
        <NSpace>
          <!-- 目标连接选择 -->
          <NSelect
            v-model:value="targetConnectionId"
            :options="connectionOptions"
            placeholder="选择目标串口"
            size="small"
            style="width: 160px"
            clearable
          />
          <NButton size="small" @click="openFile">
            <template #icon><NIcon :component="FolderOpenOutline" /></template>
            {{ t('script.open') }}
          </NButton>
          <NButton size="small" @click="saveFile">
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
          :disabled="isRunning"
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

      <NScrollbar ref="logScrollRef" class="output-content">
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

.file-list { padding: 4px 0; }

.file-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 10px;
  border-radius: 6px;
  cursor: pointer;
  font-size: var(--font-sm);
  color: var(--text-secondary);
  transition: all 0.2s;
  position: relative;
}
.file-item:hover { background: var(--border-color); }
.file-item.active { background: #e3f2fd; color: #1565c0; }

.file-label { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.modified-dot { color: #f0a020; font-size: 10px; flex-shrink: 0; }
.close-btn { opacity: 0; flex-shrink: 0; }
.file-item:hover .close-btn { opacity: 1; }

.script-info { padding: 8px 0; }
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
  tab-size: 2;
}
.code-editor:disabled {
  opacity: 0.6;
  cursor: not-allowed;
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
.output-content { flex: 1; }
.log-list { padding: 12px; }

.log-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 8px;
  border-radius: 6px;
  font-size: var(--font-xs);
  margin-bottom: 4px;
}
.log-item.info { background: var(--bg-page); color: var(--text-secondary); }
.log-item.warn { background: #fff8e1; color: #f57f17; }
.log-item.error { background: #ffebee; color: #c62828; }
.log-item.success { background: #e8f5e9; color: #2e7d32; }

.log-time { color: var(--text-muted); flex-shrink: 0; }
.log-content { flex: 1; word-break: break-all; }

.empty-log {
  text-align: center;
  color: var(--text-muted);
  padding: 32px;
  font-size: var(--font-sm);
}
</style>
