<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import {
  NButton, NSpace, NIcon, NTooltip, NDivider, NTag,
  NScrollbar, NSelect, NCheckboxGroup, NCheckbox,
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
import {
  activeConnections as serialActiveConnections, type ConnectionInfo,
} from '@/stores/serial'
import {
  activeConnections as networkActiveConnections,
} from '@/stores/network'
import {
  scriptFiles, activeScriptId, scriptLogs, scriptIsRunning, activeScript,
  selectedConnectionIds,
  scriptTemplates, clearScriptLogs, runScript, stopScript,
  newScriptFile, removeScriptFile, updateScriptContent,
} from '@/stores/script'
import { t } from '@/stores/i18n'

const message = useMessage()

// ========== 连接选择（多选） ==========

const connectionOptions = computed(() => {
  const serial = serialActiveConnections.value.map((c: ConnectionInfo) => ({
    label: `${c.config.port_name} (${c.status === 'Connected' ? '已连接' : '未连接'})`,
    value: c.connection_id,
  }))
  const net = networkActiveConnections.value.map((c) => ({
    label: `${c.config.protocol.toUpperCase()} ${c.config.host}:${c.config.port} (${c.status === 'Connected' ? '已连接' : '未连接'})`,
    value: c.connection_id,
  }))
  return [...serial, ...net]
})

// ========== 模板选择 ==========

const selectedTemplate = ref<string | null>(null)

const applyTemplate = (val: string | null) => {
  const tpl = scriptTemplates.find(t => t.value === val)
  if (!tpl) return
  const s = activeScript.value
  if (s) {
    updateScriptContent(s.id, tpl.content)
  }
}

// ========== 编辑器计算属性 ==========

const scriptContent = computed({
  get: () => activeScript.value?.content ?? '',
  set: (val: string) => {
    const s = activeScript.value
    if (s) updateScriptContent(s.id, val)
  },
})

const lineCount = computed(() => scriptContent.value.split('\n').length)
const charCount = computed(() => scriptContent.value.length)

// 日志滚动
const logScrollRef = ref()
watch(() => scriptLogs.value.length, () => {
  nextTick(() => {
    logScrollRef.value?.scrollTo({ top: 999999 })
  })
})

// ========== 文件操作 ==========

const newScript = () => {
  newScriptFile()
  selectedTemplate.value = null
  message.info(t('script.newCreated'))
}

const selectScript = (id: string) => {
  activeScriptId.value = id
}

const removeScript = (id: string) => {
  removeScriptFile(id)
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

    const existing = scriptFiles.value.find(s => s.path === filePath)
    if (existing) {
      updateScriptContent(existing.id, content)
      existing.modified = false
      activeScriptId.value = existing.id
      return
    }

    const id = `file_${Date.now()}_${Date.now()}`
    scriptFiles.value.push({ id, name, path: filePath, content, modified: false })
    activeScriptId.value = id
    selectedTemplate.value = null
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
  const s = activeScript.value
  if (s) s.modified = true
}

// ========== 脚本运行 ==========

const handleRun = () => {
  if (scriptIsRunning.value) {
    stopScript()
    message.info(t('script.stoppedMsg'))
  } else {
    runScript(selectedConnectionIds.value, scriptContent.value, (type, msg) => {
      if (type === 'success') message.success(msg)
      if (type === 'error') message.error(msg)
      if (type === 'warn') message.warning(msg)
      if (type === 'info') message.info(msg)
    })
  }
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
            :key="file.id"
            class="file-item"
            :class="{ active: file.id === activeScriptId }"
            @click="selectScript(file.id)"
          >
            <NIcon :component="DocumentOutline" size="14" />
            <span class="file-label">{{ file.name }}</span>
            <span v-if="file.modified" class="modified-dot">●</span>
            <NButton
              v-if="scriptFiles.length > 1"
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
      <div class="editor-toolbar">
        <div class="toolbar-left">
          <NIcon :component="CodeSlashOutline" size="18" />
          <span class="file-name">{{ activeScript?.name }}</span>
          <NTag v-if="activeScript?.modified" size="small" type="warning">{{ t('script.unsaved') }}</NTag>
          <NTag v-if="scriptIsRunning" size="small" type="success">{{ t('script.running') }}</NTag>
        </div>
        <NSpace>
          <NSelect
            v-model:value="selectedTemplate"
            :options="scriptTemplates.map(t => ({ label: t.label, value: t.value }))"
            placeholder="选择模板"
            size="small"
            style="width: 160px"
            @update:value="applyTemplate"
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
            :type="scriptIsRunning ? 'error' : 'primary'"
            size="small"
            @click="handleRun"
          >
            <template #icon>
              <NIcon :component="scriptIsRunning ? StopOutline : PlayOutline" />
            </template>
            {{ scriptIsRunning ? t('script.stop') : t('script.run') }}
          </NButton>
        </NSpace>
      </div>

      <div class="target-bar">
        <NCheckboxGroup v-model:value="selectedConnectionIds">
          <NSpace>
            <NCheckbox
              v-for="opt in connectionOptions"
              :key="opt.value"
              :value="opt.value"
            >
              {{ opt.label }}
            </NCheckbox>
          </NSpace>
        </NCheckboxGroup>
      </div>

      <div class="editor-container">
        <div class="line-numbers">
          <div v-for="n in lineCount" :key="n">{{ n }}</div>
        </div>
        <textarea
          v-model="scriptContent"
          @input="onContentChange"
          class="code-editor"
          spellcheck="false"
          :disabled="scriptIsRunning"
        ></textarea>
      </div>
    </main>

    <!-- 右侧输出区 -->
    <aside class="output-panel">
      <div class="panel-header">
        <div class="section-title">
          <NIcon :component="TimeOutline" size="16" />
          <span>{{ t('script.log') }}</span>
          <NTag size="small" :bordered="false">{{ scriptLogs.length }}</NTag>
        </div>
        <NButton size="tiny" quaternary @click="clearScriptLogs">
          <template #icon><NIcon :component="TrashOutline" /></template>
        </NButton>
      </div>

      <NScrollbar ref="logScrollRef" class="output-content">
        <div class="log-list">
          <div
            v-for="(log, idx) in scriptLogs"
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
          <div v-if="scriptLogs.length === 0" class="empty-log">
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

.target-bar {
  padding: 10px 16px;
  border-bottom: 1px solid var(--border-color);
  background: #fafafa;
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
