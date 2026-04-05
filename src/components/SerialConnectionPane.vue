<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import {
  NButton, NSelect, NSpace, NInput, NTag, NIcon, NTooltip, NSwitch,
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
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { readFile } from '@tauri-apps/plugin-fs'
import { t } from '@/stores/i18n'

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
const appendNewline = ref('none')

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
  if (!connectionInfo.value) return { sent: 0, received: 0 }
  return {
    sent: connectionInfo.value.bytes_sent,
    received: connectionInfo.value.bytes_received
  }
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

const newlineOptions = computed(() => [
  { label: t('serial.newlineNone'), value: 'none' },
  { label: 'LF (\\n)', value: '\n' },
  { label: 'CRLF (\\r\\n)', value: '\r\n' },
])

const formatHex = (text: string) => {
  const bytes = new TextEncoder().encode(text)
  return Array.from(bytes).map(b => b.toString(16).toUpperCase().padStart(2, '0')).join(' ')
}

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
    await sendData(props.connectionId, textToSend, hexSend.value)
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
        </div>
      </div>
    </aside>

    <!-- 右侧主区域 -->
    <main class="main-area">
      <div class="terminal-section">
        <div class="terminal-header">
          <div class="terminal-title">
            <NIcon :component="SwapHorizontalOutline" size="18" />
            <span>{{ t('serial.terminal') }}</span>
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
              <template #unchecked>{{ t('serial.text') }}</template>
            </NSwitch>
            <NSwitch v-model:value="display.autoScroll" size="small">
              <template #checked>{{ t('serial.scroll') }}</template>
              <template #unchecked>{{ t('serial.scroll') }}</template>
            </NSwitch>
            <NButton size="small" quaternary @click="clearLog">
              <template #icon><NIcon :component="TrashOutline" /></template>
              {{ t('serial.clear') }}
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
              <p>{{ t('serial.emptyHint') }}</p>
            </div>
          </div>
        </NScrollbar>
      </div>

      <div class="send-section">
        <div class="send-options">
          <NSpace align="center" :size="12">
            <NSwitch v-model:value="hexSend" size="small">
              <template #checked>{{ t('serial.hexSend') }}</template>
              <template #unchecked>{{ t('serial.textSend') }}</template>
            </NSwitch>
            <NSelect
              v-if="!hexSend"
              v-model:value="appendNewline"
              :options="newlineOptions"
              size="small"
              style="width: 120px"
            />
          </NSpace>
        </div>
        <div class="send-input">
          <NInput
            v-model:value="sendText"
            :disabled="!isConnected"
            :placeholder="hexSend ? t('serial.hexPlaceholder') : t('serial.textPlaceholder')"
            @keydown.enter="handleSend"
            clearable
          />
          <NButton
            type="primary"
            :disabled="!isConnected || !sendText.trim()"
            @click="handleSend"
          >
            <template #icon><NIcon :component="SendOutline" /></template>
            {{ t('serial.send') }}
          </NButton>
          <NButton
            :disabled="!isConnected"
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

/* 终端区域 */
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

.terminal-filter-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  background: #1e1e1e;
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

/* 发送区域 */
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
</style>
