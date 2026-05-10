<script setup lang="ts">
import { ref, computed, watch, onUnmounted, nextTick } from 'vue'
import {
  NButton, NSelect, NSpace, NInput, NInputNumber, NTag, NIcon, NTooltip, NSwitch,
  NScrollbar, NDivider, NInputGroup, NCheckboxGroup, NCheckbox,
  useMessage
} from 'naive-ui'
import {
  RefreshOutline, FlashOutline, CloseOutline, SendOutline,
  TrashOutline, SettingsOutline, PulseOutline, SwapHorizontalOutline,
  DocumentOutline
} from '@vicons/ionicons5'
import {
  availablePorts,
  refreshPorts as doRefreshPorts,
  openSerialPort,
  closeSerialPort,
  sendData,
  sendDataWithCrc,
  sendFile,
  generateConnectionId,
  getPortDisplayName,
  activeConnections,
  getConnectionLog,
  addConnectionLog,
  clearConnectionLog,
  getConnectionDisplay,
  setConnectionDisplay,
  type SerialPortConfig,
} from '@/stores/serial'
import { invoke } from '@tauri-apps/api/core'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { readFile } from '@tauri-apps/plugin-fs'
import { t } from '@/stores/i18n'
import { appConfig, saveConfig } from '@/stores/config'
import type { QuickCommand, NewlineType } from '@/stores/config'
import { formatHex } from '@/utils/hex'
import { formatRate, formatDuration } from '@/utils/format'
import ConnectionTerminal from './ConnectionTerminal.vue'
import ConnectionSendPane from './ConnectionSendPane.vue'

const props = defineProps<{
  tabId: string
  connectionId: string | null
  config: SerialPortConfig
}>()

const emit = defineEmits<{
  (e: 'update:config', value: SerialPortConfig): void
  (e: 'connect', config: SerialPortConfig, connectionId: string): void
  (e: 'disconnect'): void
}>()

const message = useMessage()

// 本地可编辑配置副本
const localConfig = ref<SerialPortConfig>({ ...props.config })

watch(() => props.config, (newConfig) => {
  localConfig.value = { ...newConfig }
}, { deep: true })

const onConfigChange = () => {
  emit('update:config', { ...localConfig.value })
}

// UI 状态
const loading = ref(false)
const hexSend = ref(false)
const sendText = ref('')
const appendNewline = ref<NewlineType>('none')
const dtrEnabled = ref(false)
const rtsEnabled = ref(false)
const selectedCrc = ref('none')

const crcOptions = [
  { label: 'None', value: 'none' },
  { label: 'Modbus', value: 'modbus' },
  { label: 'XOR-8', value: 'xor8' },
  { label: 'CRC16-CCITT', value: 'crc16-ccitt' },
  { label: 'CRC32', value: 'crc32' },
]

// 显示设置
const display = computed({
  get: () => getConnectionDisplay(props.connectionId),
  set: (val) => {
    if (props.connectionId) {
      setConnectionDisplay(props.connectionId, val)
    }
  }
})

// 终端引用
const scrollbarRef = ref()

// 计算属性
const connectionInfo = computed(() => {
  if (!props.connectionId) return null
  return activeConnections.value.find(c => c.connection_id === props.connectionId) || null
})

const isConnected = computed(() => {
  return connectionInfo.value?.status === 'Connected'
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
watch(isConnected, (connected) => {
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
  // 依赖 durationNow 以确保每秒重新计算
  durationNow.value
  return formatDuration(connectionStats.value.connectedAt)
})

// 终端日志过滤
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

const portSelectOpen = ref(false)
const frozenPortOptions = ref<{ label: string; value: string }[]>([])

const livePortOptions = computed(() =>
  availablePorts.value.map(p => ({
    label: getPortDisplayName(p),
    value: p.port_name
  }))
)

const portOptions = computed(() =>
  portSelectOpen.value ? frozenPortOptions.value : livePortOptions.value
)

function onPortSelectShow(show: boolean) {
  if (show) {
    frozenPortOptions.value = livePortOptions.value
  }
  portSelectOpen.value = show
}

const baudRateOptions = [
  { label: '9600', value: 9600 },
  { label: '19200', value: 19200 },
  { label: '38400', value: 38400 },
  { label: '57600', value: 57600 },
  { label: '115200', value: 115200 },
  { label: '230400', value: 230400 },
  { label: '460800', value: 460800 },
  { label: '921600', value: 921600 },
  { label: '1500000', value: 1500000 },
  { label: '2000000', value: 2000000 },
  { label: '3000000', value: 3000000 },
  { label: '4000000', value: 4000000 },
]

const dataBitsOptions = computed(() => [
  { label: t('serial.bits5'), value: 5 },
  { label: t('serial.bits6'), value: 6 },
  { label: t('serial.bits7'), value: 7 },
  { label: t('serial.bits8'), value: 8 },
])

const stopBitsOptions = computed(() => [
  { label: t('serial.stop1'), value: 1 },
  { label: t('serial.stop2'), value: 2 },
])

const parityOptions = computed(() => [
  { label: t('serial.parityNone'), value: 'None' },
  { label: t('serial.parityOdd'), value: 'Odd' },
  { label: t('serial.parityEven'), value: 'Even' },
])

const encodingOptions = [
  { label: 'UTF-8', value: 'utf-8' },
  { label: 'GBK', value: 'gbk' },
]

const newlineOptions = computed<{ label: string; value: NewlineType }[]>(() => [
  { label: t('serial.newlineNone'), value: 'none' },
  { label: 'LF (\\n)', value: '\n' },
  { label: 'CRLF (\\r\\n)', value: '\r\n' },
])

const terminalLogs = computed(() => {
  return props.connectionId ? getConnectionLog(props.connectionId) : []
})

// 刷新串口
const refreshPorts = async () => {
  loading.value = true
  try {
    await doRefreshPorts()
    if (availablePorts.value.length > 0 && !localConfig.value.port_name) {
      localConfig.value.port_name = availablePorts.value[0].port_name
      onConfigChange()
    }
    message.success(t('serial.foundPorts', availablePorts.value.length))
  } catch (e) {
    message.error(t('serial.refreshFail', String(e)))
  } finally {
    loading.value = false
  }
}

// 连接/断开
const toggleConnection = async () => {
  loading.value = true
  let pendingConnId: string | null = null
  try {
    if (isConnected.value && props.connectionId) {
      await closeSerialPort(props.connectionId)
      addConnectionLog(props.connectionId, {
        type: 'system',
        content: t('serial.disconnectedMsg'),
        time: new Date().toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
      })
      message.info(t('serial.portDisconnected'))
      emit('disconnect')
    } else {
      if (!localConfig.value.port_name) {
        message.warning(t('serial.selectFirst'))
        return
      }
      pendingConnId = generateConnectionId()
      await openSerialPort(pendingConnId, { ...localConfig.value })
      emit('connect', { ...localConfig.value }, pendingConnId)
      addConnectionLog(pendingConnId, {
        type: 'system',
        content: t('serial.connectedLog', localConfig.value.port_name, localConfig.value.baud_rate, localConfig.value.data_bits, localConfig.value.parity[0], localConfig.value.stop_bits),
        time: new Date().toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
      })
      message.success(t('serial.connectedMsg'))
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
    message.error(t('serial.operationFail', String(e)))
  } finally {
    loading.value = false
  }
}

// 发送
const handleSend = async () => {
  if (!sendText.value.trim() || !props.connectionId) return
  try {
    let textToSend = sendText.value
    if (!hexSend.value && appendNewline.value !== 'none') {
      textToSend += appendNewline.value
    }
    if (selectedCrc.value !== 'none') {
      await sendDataWithCrc(props.connectionId, textToSend, hexSend.value, selectedCrc.value)
    } else {
      await sendData(props.connectionId, textToSend, hexSend.value)
    }
    sendText.value = ''
  } catch (e) {
    message.error(t('serial.sendFail', String(e)))
  }
}

const handleSendFile = async () => {
  if (!props.connectionId) return
  try {
    const selected = await openDialog({ multiple: false })
    if (!selected) return
    const bytes = await readFile(selected as string)
    const sent = await sendFile(props.connectionId, bytes)
    message.success(t('serial.fileSent', sent))
  } catch (e) {
    message.error(t('serial.fileSendFail', String(e)))
  }
}

// ========== DTR / RTS 控制 ==========
const toggleDtr = async () => {
  if (!props.connectionId) return
  try {
    dtrEnabled.value = !dtrEnabled.value
    await invoke('set_serial_dtr', { connectionId: props.connectionId, level: dtrEnabled.value })
  } catch (e) {
    message.error(`DTR 设置失败: ${e}`)
    dtrEnabled.value = !dtrEnabled.value
  }
}

const toggleRts = async () => {
  if (!props.connectionId) return
  try {
    rtsEnabled.value = !rtsEnabled.value
    await invoke('set_serial_rts', { connectionId: props.connectionId, level: rtsEnabled.value })
  } catch (e) {
    message.error(`RTS 设置失败: ${e}`)
    rtsEnabled.value = !rtsEnabled.value
  }
}

// ========== 快捷命令 ==========
const quickCmdForm = ref({ name: '', content: '', isHex: false })
const showQuickCmdForm = ref(false)

const quickCommands = computed(() => appConfig.value?.serial?.quick_commands || [])

const handleQuickSend = async (cmd: QuickCommand) => {
  if (!props.connectionId) return
  try {
    let textToSend = cmd.content
    if (!cmd.is_hex && cmd.append_newline && cmd.append_newline !== 'none') {
      textToSend += cmd.append_newline
    }
    await sendData(props.connectionId, textToSend, cmd.is_hex)
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
    append_newline: appendNewline.value,
  }
  if (appConfig.value) {
    appConfig.value.serial.quick_commands = [...quickCommands.value, newCmd]
    await saveConfig()
  }
}

const removeQuickCommand = async (index: number) => {
  if (appConfig.value) {
    const newList = quickCommands.value.filter((_, i) => i !== index)
    appConfig.value.serial.quick_commands = newList
    await saveConfig()
  }
}

const clearLog = () => {
  if (props.connectionId) {
    clearConnectionLog(props.connectionId)
  }
}

// 自动滚动到底部
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
        <div class="status-indicator" :class="{ connected: isConnected }">
          <div class="status-dot"></div>
          <span class="status-text">{{ isConnected ? t('serial.connected') : t('serial.disconnected') }}</span>
        </div>
        <div v-if="isConnected && connectionInfo" class="connection-info">
          <span>{{ connectionInfo.config.port_name }}</span>
          <span class="baud">{{ connectionInfo.config.baud_rate }} bps</span>
        </div>
      </div>

      <NDivider style="margin: 16px 0" />

      <div class="config-section">
        <div class="section-title">
          <NIcon :component="SettingsOutline" size="16" />
          <span>{{ t('serial.config') }}</span>
        </div>

        <div class="config-item">
          <label>{{ t('serial.port') }}</label>
          <NInputGroup>
            <NSelect
              v-model:value="localConfig.port_name"
              :options="portOptions"
              :disabled="isConnected"
              :virtual-scroll="false"
              :placeholder="t('serial.selectPort')"
              size="small"
              style="flex: 1"
              @update:show="onPortSelectShow"
              @update:value="onConfigChange"
            />
            <NTooltip>
              <template #trigger>
                <NButton
                  size="small"
                  :loading="loading"
                  :disabled="isConnected"
                  @click="refreshPorts"
                >
                  <template #icon>
                    <NIcon :component="RefreshOutline" />
                  </template>
                </NButton>
              </template>
              {{ t('serial.refreshPorts') }}
            </NTooltip>
          </NInputGroup>
        </div>

        <div class="config-item">
          <label>{{ t('serial.baudRate') }}</label>
          <NSelect
            v-model:value="localConfig.baud_rate"
            :options="baudRateOptions"
            :disabled="isConnected"
            filterable
            size="small"
            @update:value="onConfigChange"
          />
        </div>

        <div class="config-row">
          <div class="config-item half">
            <label>{{ t('serial.dataBits') }}</label>
            <NSelect
              v-model:value="localConfig.data_bits"
              :options="dataBitsOptions"
              :disabled="isConnected"
              size="small"
              @update:value="onConfigChange"
            />
          </div>
          <div class="config-item half">
            <label>{{ t('serial.stopBits') }}</label>
            <NSelect
              v-model:value="localConfig.stop_bits"
              :options="stopBitsOptions"
              :disabled="isConnected"
              size="small"
              @update:value="onConfigChange"
            />
          </div>
        </div>

        <div class="config-item">
          <label>{{ t('serial.parity') }}</label>
          <NSelect
            v-model:value="localConfig.parity"
            :options="parityOptions"
            :disabled="isConnected"
            size="small"
            @update:value="onConfigChange"
          />
        </div>

        <NDivider style="margin: 16px 0 12px" />

        <div class="config-item" style="display: flex; align-items: center; gap: 8px; margin-bottom: 12px;">
          <NSwitch v-model:value="localConfig.auto_reconnect" size="small" @update:value="onConfigChange" />
          <span style="font-size: 13px;">{{ t('serial.autoReconnect') }}</span>
        </div>
        <div v-if="localConfig.auto_reconnect" class="config-row">
          <div class="config-item half">
            <label style="font-size: 12px;">{{ t('serial.reconnectInterval') }}</label>
            <NInputNumber
              v-model:value="localConfig.reconnect_interval_ms"
              :min="100"
              :step="100"
              :disabled="isConnected"
              size="small"
              @update:value="onConfigChange"
            />
          </div>
          <div class="config-item half">
            <label style="font-size: 12px;">{{ t('serial.maxReconnectAttempts') }}</label>
            <NInputNumber
              v-model:value="localConfig.max_reconnect_attempts"
              :min="1"
              :max="20"
              :disabled="isConnected"
              size="small"
              @update:value="onConfigChange"
            />
          </div>
        </div>
      </div>

      <NButton
        :type="isConnected ? 'error' : 'primary'"
        :loading="loading"
        :disabled="!localConfig.port_name && !isConnected"
        block
        size="large"
        @click="toggleConnection"
        style="margin-top: 20px"
      >
        <template #icon>
          <NIcon :component="isConnected ? CloseOutline : FlashOutline" />
        </template>
        {{ isConnected ? t('serial.disconnect') : t('serial.connect') }}
      </NButton>

      <!-- DTR / RTS 控制 -->
      <div v-if="isConnected" class="signal-controls">
        <NSpace justify="center">
          <NButton size="small" :type="dtrEnabled ? 'primary' : 'default'" @click="toggleDtr">
            DTR {{ dtrEnabled ? 'ON' : 'OFF' }}
          </NButton>
          <NButton size="small" :type="rtsEnabled ? 'primary' : 'default'" @click="toggleRts">
            RTS {{ rtsEnabled ? 'ON' : 'OFF' }}
          </NButton>
        </NSpace>
      </div>

      <div v-if="isConnected" class="stats-section">
        <NDivider style="margin: 20px 0 16px" />
        <div class="section-title">
          <NIcon :component="PulseOutline" size="16" />
          <span>{{ t('serial.statistics') }}</span>
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
        :title="t('serial.terminal')"
        :log-count="terminalLogs.length"
        :empty-hint="t('serial.emptyHint')"
        :clear-label="t('serial.clear')"
        :search-placeholder="t('serial.searchLog')"
        :encoding-options="encodingOptions"
        @clear="clearLog"
      />
      <ConnectionSendPane
        :is-connected="isConnected"
        :send-disabled="!isConnected"
        v-model:hex-send="hexSend"
        v-model:send-text="sendText"
        v-model:append-newline="appendNewline"
        v-model:selected-crc="selectedCrc"
        :show-newline="true"
        :show-crc="true"
        :show-file="true"
        :newline-options="newlineOptions"
        :crc-options="crcOptions"
        :quick-commands="quickCommands"
        :placeholder-hex="t('serial.hexPlaceholder')"
        :placeholder-text="t('serial.textPlaceholder')"
        :send-label="t('serial.send')"
        :file-label="t('serial.sendFile')"
        :add-quick-cmd-label="t('serial.addQuickCmd')"
        :cmd-name-placeholder="t('serial.cmdName')"
        :cmd-content-placeholder="t('serial.cmdContent')"
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

/* 左侧配置面板 */
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

.connection-info .baud {
  color: var(--text-muted);
}

/* 配置区 */
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

.config-row {
  display: flex;
  gap: 12px;
}

.config-item.half {
  flex: 1;
}

/* 统计 */
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

/* 右侧主区域 */
.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-width: 0;
}

</style>
