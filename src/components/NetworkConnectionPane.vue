<script setup lang="ts">
import { ref, computed, watch, onUnmounted, nextTick } from 'vue'
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
import type { QuickCommand, NewlineType } from '@/stores/config'
import { formatHex } from '@/utils/hex'
import { formatRate, formatDuration } from '@/utils/format'
import ConnectionTerminal from './ConnectionTerminal.vue'
import ConnectionSendPane from './ConnectionSendPane.vue'
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
  if (!connectionInfo.value) {
    return { sent: 0, received: 0, txRate: 0, rxRate: 0, connectedAt: null }
  }
  return {
    sent: connectionInfo.value.bytes_sent,
    received: connectionInfo.value.bytes_received,
    txRate: connectionInfo.value.tx_rate,
    rxRate: connectionInfo.value.rx_rate,
    connectedAt: connectionInfo.value.connected_at,
  }
})

// 连接时长刷新定时器
const durationNow = ref(Date.now())
let durationTimer: number | null = null
watch(isActive, (connected) => {
  if (connected) {
    durationTimer = window.setInterval(() => { durationNow.value = Date.now() }, 1000)
  } else if (durationTimer !== null) {
    clearInterval(durationTimer)
    durationTimer = null
  }
}, { immediate: true })
onUnmounted(() => {
  if (durationTimer !== null) clearInterval(durationTimer)
})

const connectionDuration = computed(() => {
  durationNow.value
  return formatDuration(connectionStats.value.connectedAt)
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

const addQuickCommand = async (...args: [string, string, boolean]) => {
  const [name, content, isHex] = args
  if (!name.trim() || !content.trim()) return
  const newCmd: QuickCommand = {
    name: name.trim(),
    content: content,
    is_hex: isHex,
    append_newline: 'none' as NewlineType,
  }
  if (appConfig.value) {
    appConfig.value.network.quick_commands = [...quickCommands.value, newCmd]
    await saveConfig()
  }
}

const removeQuickCommand = async (index: number) => {
  if (appConfig.value) {
    const newList = quickCommands.value.filter((_, i) => i !== index)
    appConfig.value.network.quick_commands = newList
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
          <span>{{ t('network.config') }}</span>
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
          <span>{{ t('network.statistics') }}</span>
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
          <div class="stat-item">
            <span class="stat-label">↑ TX/s</span>
            <span class="stat-value tx">{{ formatRate(connectionStats.txRate) }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">↓ RX/s</span>
            <span class="stat-value rx">{{ formatRate(connectionStats.rxRate) }}</span>
          </div>
          <div class="stat-item" style="grid-column: span 2;">
            <span class="stat-label">时长</span>
            <span class="stat-value">{{ connectionDuration }}</span>
          </div>
        </div>
      </div>
    </aside>

    <!-- 右侧主区域 -->
    <main class="main-area">
      <ConnectionTerminal
        :logs="terminalLogs"
        :title="t('network.terminal')"
        :log-count="terminalLogs.length"
        :empty-hint="t('network.emptyHint')"
        :clear-label="t('network.clear')"
        :search-placeholder="t('serial.searchLog')"
        :encoding-options="encodingOptions"
        @clear="clearLog"
      />
      <ConnectionSendPane
        :is-connected="isActive"
        :send-disabled="!isActive || (isServer && peers.length > 0 && !currentSelectedPeer)"
        v-model:hex-send="hexSend"
        v-model:send-text="sendText"
        :show-newline="false"
        :show-crc="false"
        :show-file="true"
        :quick-commands="quickCommands"
        placeholder-hex="输入十六进制数据，如: 01 02 03 FF"
        placeholder-text="输入要发送的数据..."
        :send-label="t('network.send')"
        file-label="发送文件"
        add-quick-cmd-label="+ 添加快捷命令"
        cmd-name-placeholder="命令名称"
        cmd-content-placeholder="命令内容"
        @send="handleSend"
        @send-file="handleSendFile"
        @add-quick-command="(name, content, isHex) => addQuickCommand(name, content, isHex)"
        @remove-quick-command="removeQuickCommand"
        @quick-send="handleQuickSend"
      />
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

</style>
