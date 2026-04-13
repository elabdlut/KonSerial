<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import {
  NButton, NSelect, NSpace, NInput, NInputNumber,
  NIcon, NSwitch, NScrollbar, NDivider, NTag,
  NCheckboxGroup, NCheckbox,
  useMessage
} from 'naive-ui'
import {
  FlashOutline, CloseOutline, SendOutline,
  TrashOutline, SettingsOutline, SwapHorizontalOutline,
  GlobeOutline, DocumentOutline
} from '@vicons/ionicons5'
import {
  openNetworkConnection,
  closeNetworkConnection,
  sendNetworkData,
  sendNetworkFile,
  activeConnections,
  getConnectionLog,
  addConnectionLog,
  clearConnectionLog,
  getConnectionDisplay,
  setConnectionDisplay,
  generateConnectionId,
  connectionPeers,
  selectedPeer,
  type NetConnectionConfig,
} from '@/stores/network'
import { t } from '@/stores/i18n'
import { appConfig, saveConfig } from '@/stores/config'
import type { QuickCommand } from '@/stores/config'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { readFile } from '@tauri-apps/plugin-fs'

const props = defineProps<{
  tabId: string
  connectionId: string | null
  config: NetConnectionConfig
}>()

const emit = defineEmits<{
  (e: 'update:config', value: NetConnectionConfig): void
  (e: 'connect', config: NetConnectionConfig, connectionId: string): void
  (e: 'disconnect'): void
}>()

const message = useMessage()

const localConfig = ref<NetConnectionConfig>({ ...props.config })

watch(() => props.config, (newConfig) => {
  localConfig.value = { ...newConfig }
}, { deep: true })

const onConfigChange = () => {
  emit('update:config', { ...localConfig.value })
}

const loading = ref(false)
const hexSend = ref(false)
const sendText = ref('')

const display = computed({
  get: () => getConnectionDisplay(props.connectionId),
  set: (val) => {
    if (props.connectionId) {
      setConnectionDisplay(props.connectionId, val)
    }
  }
})

const scrollbarRef = ref()

const connectionInfo = computed(() => {
  if (!props.connectionId) return null
  return activeConnections.value.find(c => c.connection_id === props.connectionId) || null
})

const isServer = computed(() => localConfig.value.protocol.endsWith('_server'))

const isActive = computed(() => {
  const status = connectionInfo.value?.status
  return status === 'Connected' || status === 'Listening'
})

const connectionStats = computed(() => {
  if (!connectionInfo.value) return { sent: 0, received: 0 }
  return {
    sent: connectionInfo.value.bytes_sent,
    received: connectionInfo.value.bytes_received
  }
})

const terminalLogs = computed(() => {
  return props.connectionId ? getConnectionLog(props.connectionId) : []
})

const logFilterText = ref('')
const logFilterTypes = ref<string[]>(['tx', 'rx', 'system', 'error'])

const filteredTerminalLogs = computed(() => {
  const text = logFilterText.value.trim().toLowerCase()
  return terminalLogs.value.filter((item) => {
    if (!logFilterTypes.value.includes(item.type)) return false
    if (!text) return true
    return (
      item.content.toLowerCase().includes(text) ||
      item.time.toLowerCase().includes(text)
    )
  })
})

const protoOptions = [
  { label: 'TCP', value: 'tcp' },
  { label: 'UDP', value: 'udp' },
  { label: 'WebSocket', value: 'ws' },
  { label: 'MQTT', value: 'mqtt' },
  { label: t('network.tcpServer'), value: 'tcp_server' },
  { label: t('network.udpServer'), value: 'udp_server' },
]

const encodingOptions = [
  { label: 'UTF-8', value: 'utf-8' },
  { label: 'GBK', value: 'gbk' },
]

const peers = computed(() => {
  if (!props.connectionId) return []
  return connectionPeers.value[props.connectionId] || []
})

const currentSelectedPeer = computed({
  get: () => {
    if (!props.connectionId) return null
    return selectedPeer.value[props.connectionId] || null
  },
  set: (val) => {
    if (props.connectionId) {
      selectedPeer.value[props.connectionId] = val
    }
  }
})

const formatHex = (text: string) => {
  const bytes = new TextEncoder().encode(text)
  return Array.from(bytes).map(b => b.toString(16).toUpperCase().padStart(2, '0')).join(' ')
}

const toggleConnection = async () => {
  loading.value = true
  let pendingConnId: string | null = null
  try {
    if (isActive.value && props.connectionId) {
      await closeNetworkConnection(props.connectionId)
      if (isServer.value) {
        addConnectionLog(props.connectionId, {
          type: 'system',
          content: t('network.serverStopped'),
          time: new Date().toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
        })
      } else {
        addConnectionLog(props.connectionId, {
          type: 'system',
          content: '网络连接已断开',
          time: new Date().toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
        })
      }
      emit('disconnect')
    } else {
      if (!localConfig.value.host) {
        message.warning('请输入目标地址')
        return
      }
      pendingConnId = generateConnectionId()
      await openNetworkConnection(pendingConnId, { ...localConfig.value })
      emit('connect', { ...localConfig.value }, pendingConnId)
      if (isServer.value) {
        addConnectionLog(pendingConnId, {
          type: 'system',
          content: t('network.serverStarted', localConfig.value.protocol.toUpperCase(), localConfig.value.host, String(localConfig.value.port)),
          time: new Date().toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
        })
      } else {
        addConnectionLog(pendingConnId, {
          type: 'system',
          content: `已连接 ${localConfig.value.protocol.toUpperCase()}://${localConfig.value.host}:${localConfig.value.port}`,
          time: new Date().toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
        })
        message.success('网络连接成功')
      }
    }
  } catch (e) {
    const errConnId = props.connectionId || pendingConnId
    if (errConnId) {
      addConnectionLog(errConnId, {
        type: 'error',
        content: String(e),
        time: new Date().toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
      })
    }
    message.error(`操作失败: ${String(e)}`)
  } finally {
    loading.value = false
  }
}

const handleSend = async () => {
  if (!sendText.value.trim() || !props.connectionId) return
  try {
    const peerId = isServer.value ? (currentSelectedPeer.value || undefined) : undefined
    await sendNetworkData(props.connectionId, sendText.value, hexSend.value, peerId)
    sendText.value = ''
  } catch (e) {
    message.error(`发送失败: ${String(e)}`)
  }
}

const handleSendFile = async () => {
  if (!props.connectionId) return
  try {
    const selected = await openDialog({ multiple: false })
    if (!selected) return
    const bytes = await readFile(selected as string)
    const peerId = isServer.value ? (currentSelectedPeer.value || undefined) : undefined
    const sent = await sendNetworkFile(props.connectionId, bytes, peerId)
    message.success(t('serial.fileSent', sent))
  } catch (e) {
    message.error(t('serial.fileSendFail', String(e)))
  }
}

// ========== 快捷命令 ==========
const quickCmdForm = ref({ name: '', content: '', isHex: false })
const showQuickCmdForm = ref(false)

const quickCommands = computed(() => appConfig.value?.network?.quick_commands || [])

const handleQuickSend = async (cmd: QuickCommand) => {
  if (!props.connectionId) return
  try {
    await sendNetworkData(props.connectionId, cmd.content, cmd.is_hex)
  } catch (e) {
    message.error(t('serial.sendFail', String(e)))
  }
}

const addQuickCommand = async () => {
  if (!quickCmdForm.value.name.trim() || !quickCmdForm.value.content.trim()) return
  const list = appConfig.value?.network?.quick_commands || []
  list.push({
    name: quickCmdForm.value.name.trim(),
    content: quickCmdForm.value.content,
    is_hex: quickCmdForm.value.isHex,
    append_newline: 'none',
  })
  if (appConfig.value) {
    appConfig.value.network.quick_commands = [...list]
    await saveConfig()
  }
  quickCmdForm.value = { name: '', content: '', isHex: false }
  showQuickCmdForm.value = false
}

const removeQuickCommand = async (index: number) => {
  const list = appConfig.value?.network?.quick_commands || []
  list.splice(index, 1)
  if (appConfig.value) {
    appConfig.value.network.quick_commands = [...list]
    await saveConfig()
  }
}

const clearLog = () => {
  if (props.connectionId) {
    clearConnectionLog(props.connectionId)
  }
}

watch(() => filteredTerminalLogs.value.length, () => {
  if (display.value.autoScroll) {
    nextTick(() => {
      scrollbarRef.value?.scrollTo({ top: 999999 })
    })
  }
})
</script>

<template>
  <div class="pane-root">
    <!-- 左侧配置区 -->
    <aside class="config-panel">
      <div class="status-section">
        <div class="status-indicator" :class="{ connected: isActive }">
          <div class="status-dot"></div>
          <span class="status-text">{{
            connectionInfo?.status === 'Listening'
              ? t('network.listening')
              : isActive
                ? t('network.connected')
                : t('network.disconnected')
          }}</span>
        </div>
        <div v-if="isActive && connectionInfo" class="connection-info">
          <span>{{ connectionInfo.config.protocol.toUpperCase() }}</span>
          <span class="host">{{ connectionInfo.config.host }}:{{ connectionInfo.config.port }}</span>
        </div>
      </div>

      <NDivider style="margin: 16px 0" />

      <div class="config-section">
        <div class="section-title">
          <NIcon :component="SettingsOutline" size="16" />
          <span>网络配置</span>
        </div>

        <div class="config-item">
          <label>{{ t('network.protocol') }}</label>
          <NSelect
            v-model:value="localConfig.protocol"
            :options="protoOptions"
            :disabled="isActive"
            size="small"
            @update:value="onConfigChange"
          />
        </div>

        <div class="config-item">
          <label>{{ t('network.host') }}</label>
          <NInput
            v-model:value="localConfig.host"
            :disabled="isActive"
            placeholder="127.0.0.1"
            size="small"
            @update:value="onConfigChange"
          />
        </div>

        <div class="config-item">
          <label>{{ t('network.port') }}</label>
          <NInputNumber
            v-model:value="localConfig.port"
            :min="1"
            :max="65535"
            :disabled="isActive"
            size="small"
            @update:value="onConfigChange"
          />
        </div>

        <div v-if="isServer && isActive && peers.length > 0" class="config-item">
          <label>{{ t('network.targetPeer') }}</label>
          <NSelect
            v-model:value="currentSelectedPeer"
            :options="peers.map(p => ({ label: p, value: p }))"
            :placeholder="t('network.selectPeer')"
            size="small"
          />
        </div>

        <div v-if="localConfig.protocol === 'ws'" class="config-item">
          <label>{{ t('network.path') }}</label>
          <NInput
            v-model:value="localConfig.path"
            :disabled="isActive"
            placeholder="/ws"
            size="small"
            @update:value="onConfigChange"
          />
        </div>

        <div v-if="localConfig.protocol === 'mqtt'" class="config-item">
          <label>{{ t('network.topic') }}</label>
          <NInput
            v-model:value="localConfig.topic"
            :disabled="isActive"
            placeholder="test/topic"
            size="small"
            @update:value="onConfigChange"
          />
        </div>
      </div>

      <NButton
        :type="isActive ? 'error' : 'primary'"
        :loading="loading"
        :disabled="!localConfig.host && !isActive"
        block
        size="large"
        @click="toggleConnection"
        style="margin-top: 20px"
      >
        <template #icon>
          <NIcon :component="isActive ? CloseOutline : FlashOutline" />
        </template>
        {{ isActive ? t('network.disconnect') : t('network.connect') }}
      </NButton>

      <div v-if="isActive" class="stats-section">
        <NDivider style="margin: 20px 0 16px" />
        <div class="section-title">
          <NIcon :component="GlobeOutline" size="16" />
          <span>数据统计</span>
        </div>
        <div class="stats-grid">
          <div class="stat-item">
            <span class="stat-label">TX</span>
            <span class="stat-value tx">{{ connectionStats.sent }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">RX</span>
            <span class="stat-value rx">{{ connectionStats.received }}</span>
          </div>
        </div>
      </div>
    </aside>

    <!-- 右侧主区域 -->
    <main class="main-area">
      <div class="terminal-section">
        <div class="terminal-header">
          <div class="terminal-title">
            <NIcon :component="SwapHorizontalOutline" size="18" />
            <span>{{ t('network.terminal') }}</span>
            <NTag size="small" :bordered="false" type="info">{{ terminalLogs.length }}</NTag>
          </div>
          <NSpace align="center" :size="8">
            <NSelect
              v-model:value="display.encoding"
              :options="encodingOptions"
              size="small"
              style="width: 90px"
            />
            <NSwitch v-model:value="display.hexDisplay" size="small">
              <template #checked>HEX</template>
              <template #unchecked>文本</template>
            </NSwitch>
            <NSwitch v-model:value="display.autoScroll" size="small">
              <template #checked>滚动</template>
              <template #unchecked>滚动</template>
            </NSwitch>
            <NButton size="small" quaternary @click="clearLog">
              <template #icon><NIcon :component="TrashOutline" /></template>
              {{ t('network.clear') }}
            </NButton>
          </NSpace>
        </div>

        <div class="terminal-filter-bar">
          <NInput
            v-model:value="logFilterText"
            :placeholder="t('serial.searchLog')"
            size="tiny"
            clearable
            style="width: 140px"
          />
          <NCheckboxGroup v-model:value="logFilterTypes" size="small">
            <NSpace :size="8">
              <NCheckbox value="tx" label="TX" />
              <NCheckbox value="rx" label="RX" />
              <NCheckbox value="system" label="SYS" />
              <NCheckbox value="error" label="ERR" />
            </NSpace>
          </NCheckboxGroup>
          <NTag size="small" :bordered="false" type="info">{{ filteredTerminalLogs.length }} / {{ terminalLogs.length }}</NTag>
        </div>

        <NScrollbar ref="scrollbarRef" class="terminal-content">
          <div class="terminal-body">
            <div
              v-for="(item, idx) in filteredTerminalLogs"
              :key="idx"
              class="log-line"
              :class="item.type"
            >
              <span class="log-time">{{ item.time }}</span>
              <span class="log-type">
                {{ item.type === 'tx' ? 'TX' : item.type === 'rx' ? 'RX' : item.type === 'system' ? 'SYS' : 'ERR' }}
              </span>
              <span class="log-content">
                {{ display.hexDisplay ? formatHex(item.content) : item.content }}
              </span>
            </div>
            <div v-if="filteredTerminalLogs.length === 0" class="terminal-empty">
              <NIcon :component="SwapHorizontalOutline" size="40" />
              <p>{{ t('network.emptyHint') }}</p>
            </div>
          </div>
        </NScrollbar>
      </div>

      <div class="send-section">
        <div class="send-options">
          <NSpace align="center" :size="12">
            <NSwitch v-model:value="hexSend" size="small">
              <template #checked>{{ t('network.hexSend') }}</template>
              <template #unchecked>{{ t('network.textSend') }}</template>
            </NSwitch>
          </NSpace>
        </div>

        <!-- 快捷命令 -->
        <div class="quick-commands-bar">
          <div v-if="!showQuickCmdForm" class="quick-commands-list">
            <NButton
              v-for="(cmd, idx) in quickCommands"
              :key="idx"
              size="tiny"
              quaternary
              type="info"
              :disabled="!isActive"
              @click="handleQuickSend(cmd)"
            >
              {{ cmd.name }}
              <template #icon>
                <NIcon :component="CloseOutline" @click.stop="removeQuickCommand(idx)" />
              </template>
            </NButton>
            <NButton size="tiny" text @click="showQuickCmdForm = true">
              + {{ t('serial.addQuickCmd') }}
            </NButton>
          </div>
          <div v-else class="quick-command-form">
            <NInput
              v-model:value="quickCmdForm.name"
              :placeholder="t('serial.cmdName')"
              size="tiny"
              style="width: 100px"
            />
            <NInput
              v-model:value="quickCmdForm.content"
              :placeholder="t('serial.cmdContent')"
              size="tiny"
              style="flex: 1"
            />
            <NSwitch v-model:value="quickCmdForm.isHex" size="small">
              <template #checked>HEX</template>
              <template #unchecked>TXT</template>
            </NSwitch>
            <NButton size="tiny" @click="addQuickCommand">{{ t('serial.addQuickCmd') }}</NButton>
            <NButton size="tiny" text @click="showQuickCmdForm = false">取消</NButton>
          </div>
        </div>

        <div class="send-input">
          <NInput
            v-model:value="sendText"
            :disabled="!isActive || (isServer && peers.length > 0 && !currentSelectedPeer)"
            :placeholder="hexSend ? '输入十六进制数据，如: 01 02 03 FF' : '输入要发送的数据...'"
            @keydown.enter="handleSend"
            clearable
          />
          <NButton
            type="primary"
            :disabled="!isActive || !sendText.trim() || (isServer && peers.length > 0 && !currentSelectedPeer)"
            @click="handleSend"
          >
            <template #icon><NIcon :component="SendOutline" /></template>
            {{ t('network.send') }}
          </NButton>
          <NButton
            :disabled="!isActive || (isServer && peers.length > 0 && !currentSelectedPeer)"
            @click="handleSendFile"
          >
            <template #icon><NIcon :component="DocumentOutline" /></template>
            {{ t('serial.sendFile') }}
          </NButton>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.pane-root {
  display: flex;
  height: 100%;
  gap: 16px;
  min-width: 0;
}

.config-panel {
  width: 280px;
  flex-shrink: 0;
  background: var(--bg-card);
  border-radius: 12px;
  padding: 20px;
  box-shadow: var(--shadow-card);
  display: flex;
  flex-direction: column;
}

.status-section {
  text-align: center;
}

.status-indicator {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: var(--bg-page);
  border-radius: 20px;
  transition: all 0.3s;
}

.status-indicator.connected {
  background: #e8f5e9;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #bbb;
}

.status-indicator.connected .status-dot {
  background: #4caf50;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.status-text {
  font-size: var(--font-base);
  font-weight: 500;
  color: var(--text-secondary);
}

.status-indicator.connected .status-text {
  color: #2e7d32;
}

.connection-info {
  margin-top: 12px;
  font-size: var(--font-sm);
  color: var(--text-secondary);
  display: flex;
  justify-content: center;
  gap: 8px;
}

.connection-info .host {
  color: var(--text-muted);
}

.config-section {
  flex: 1;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-sm);
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 16px;
}

.config-item {
  margin-bottom: 12px;
}

.config-item label {
  display: block;
  font-size: var(--font-xs);
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.stats-section {
  margin-top: auto;
}

.stats-grid {
  display: flex;
  gap: 12px;
}

.stat-item {
  flex: 1;
  background: var(--bg-page);
  border-radius: 8px;
  padding: 12px;
  text-align: center;
}

.stat-label {
  display: block;
  font-size: var(--font-2xs);
  color: var(--text-muted);
  margin-bottom: 4px;
}

.stat-value {
  font-size: var(--font-xl);
  font-weight: 600;
  font-family: 'SF Mono', Monaco, monospace;
}

.stat-value.tx { color: #1976d2; }
.stat-value.rx { color: #388e3c; }

.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-width: 0;
}

.terminal-section {
  flex: 1;
  background: var(--bg-terminal);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.terminal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: #252526;
  border-bottom: 1px solid #333;
}

.terminal-title {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #ccc;
  font-size: var(--font-base);
  font-weight: 500;
}

.terminal-content {
  flex: 1;
  min-height: 0;
}

.terminal-body {
  padding: 12px 16px;
  min-height: 100%;
}

.log-line {
  display: flex;
  gap: 12px;
  padding: 4px 0;
  font-family: 'SF Mono', Monaco, Consolas, monospace;
  font-size: var(--app-font-size, 13px);
  line-height: 1.5;
}

.log-time {
  color: #666;
  flex-shrink: 0;
}

.log-type {
  width: 32px;
  flex-shrink: 0;
  font-weight: 600;
}

.log-line.tx .log-type { color: #64b5f6; }
.log-line.rx .log-type { color: #81c784; }
.log-line.system .log-type { color: #ffb74d; }
.log-line.error .log-type { color: #e57373; }

.log-content {
  color: #d4d4d4;
  word-break: break-all;
}

.terminal-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: #555;
  gap: 12px;
}

.terminal-empty p {
  font-size: var(--font-base);
}

.send-section {
  background: var(--bg-card);
  border-radius: 12px;
  padding: 16px;
  box-shadow: var(--shadow-card);
}

.send-options {
  margin-bottom: 12px;
}

.send-input {
  display: flex;
  gap: 12px;
}

.send-input :deep(.n-input) {
  flex: 1;
}

.terminal-filter-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  background: #1e1e1e;
  border-bottom: 1px solid #333;
}

.quick-commands-bar {
  margin-bottom: 12px;
}

.quick-commands-list {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.quick-command-form {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
