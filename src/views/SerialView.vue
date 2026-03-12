<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue'
import {
  NButton, NSelect, NSpace, NInput, NTag, NIcon, NTooltip, NSwitch,
  NScrollbar, NDivider, NInputGroup,
  useMessage
} from 'naive-ui'
import {
  RefreshOutline, FlashOutline, CloseOutline, SendOutline,
  TrashOutline, SettingsOutline, PulseOutline, SwapHorizontalOutline
} from '@vicons/ionicons5'
import {
  availablePorts,
  currentConnectionId,
  currentConnection,
  refreshPorts as doRefreshPorts,
  openSerialPort,
  closeSerialPort,
  sendData as doSendData,
  updateGlobalInfo,
  generateConnectionId,
  addReceivedData,
  getPortDisplayName,
  onSerialData,
  startStatusPolling,
  stopStatusPolling,
  type SerialPortConfig,
} from '@/stores/serial'
import { maxBufferSize } from '@/stores/settings'
import { t } from '@/stores/i18n'

const message = useMessage()

// 串口配置
const portName = ref<string | null>(null)
const baudRate = ref(115200)
const dataBits = ref(8)
const stopBits = ref(1)
const parity = ref('None')
const flowControl = ref('None')

// UI 状态
const loading = ref(false)
const hexSend = ref(false)
const hexDisplay = ref(false)
const sendText = ref('')
const encoding = ref('utf-8')
const appendNewline = ref('none')
const autoScroll = ref(true)
const scrollbarRef = ref()

interface LogEntry {
  type: string
  content: string
  rawBytes?: number[]
  time: string
}
const receivedData = ref<LogEntry[]>([])

// 计算属性
const isConnected = computed(() => {
  if (!currentConnection.value) return false
  return currentConnection.value.status === 'Connected'
})

const connectionStats = computed(() => {
  if (!currentConnection.value) return { sent: 0, received: 0 }
  return {
    sent: currentConnection.value.bytes_sent,
    received: currentConnection.value.bytes_received
  }
})

const portSelectOpen = ref(false)
const frozenPortOptions = ref<{ label: string; value: string }[]>([])

const livePortOptions = computed(() => 
  availablePorts.value.map(p => ({ 
    label: getPortDisplayName(p), 
    value: p.port_name 
  }))
)

// 下拉框打开时冻结选项，避免轮询刷新导致列表跳动
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

const formatHex = (bytes: number[]) => {
  return bytes.map(b => b.toString(16).toUpperCase().padStart(2, '0')).join(' ')
}

// 刷新串口
const refreshPorts = async () => {
  loading.value = true
  try {
    await doRefreshPorts()
    if (availablePorts.value.length > 0 && !portName.value) {
      portName.value = availablePorts.value[0].port_name
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
  try {
    if (isConnected.value && currentConnectionId.value) {
      await closeSerialPort(currentConnectionId.value)
      addLog('system', t('serial.disconnectedMsg'))
      message.info(t('serial.portDisconnected'))
    } else {
      if (!portName.value) {
        message.warning(t('serial.selectFirst'))
        return
      }
      const config: SerialPortConfig = {
        port_name: portName.value,
        baud_rate: baudRate.value,
        data_bits: dataBits.value,
        stop_bits: stopBits.value,
        parity: parity.value,
        flow_control: flowControl.value,
        timeout_ms: 100,
      }
      const connId = generateConnectionId()
      await openSerialPort(connId, config)
      addLog('system', t('serial.connectedLog', portName.value, baudRate.value, dataBits.value, parity.value[0], stopBits.value))
      message.success(t('serial.connectedMsg'))
    }
  } catch (e) {
    addLog('error', `${e}`)
    message.error(t('serial.operationFail', String(e)))
  } finally {
    loading.value = false
  }
}

// 发送
const handleSend = async () => {
  if (!sendText.value.trim() || !currentConnectionId.value) return
  try {
    let textToSend = sendText.value
    if (!hexSend.value && appendNewline.value !== 'none') {
      textToSend += appendNewline.value
    }
    await doSendData(currentConnectionId.value, textToSend, hexSend.value)
    addLog('tx', sendText.value)
    sendText.value = ''
  } catch (e) {
    message.error(t('serial.sendFail', String(e)))
  }
}

const addLog = (type: string, content: string, rawBytes?: number[]) => {
  const now = new Date()
  const time = now.toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })
  receivedData.value.push({ type, content, time, rawBytes })
  
  // 超过缓冲区上限时裁剪旧条目
  const max = maxBufferSize.value
  if (receivedData.value.length > max) {
    receivedData.value = receivedData.value.slice(-max)
  }
  
  // 如果是接收的数据，同步到全局缓存供波形图使用
  if (type === 'rx' && currentConnectionId.value) {
    addReceivedData(currentConnectionId.value, content)
  }
  // 自动滚动到底部
  if (autoScroll.value) {
    nextTick(() => {
      scrollbarRef.value?.scrollTo({ top: 999999 })
    })
  }
}

const clearLog = () => {
  receivedData.value = []
}

// 注册串口数据回调，接收后端推送的数据
let unsubscribeData: (() => void) | null = null

onMounted(async () => {
  unsubscribeData = onSerialData((connId, rawData) => {
    if (connId === currentConnectionId.value) {
      const decoder = new TextDecoder(encoding.value)
      const text = decoder.decode(rawData)
      addLog('rx', text, Array.from(rawData))
    }
  })
  startStatusPolling(1000)
  await updateGlobalInfo()
  await refreshPorts()
})

onUnmounted(() => {
  unsubscribeData?.()
  stopStatusPolling()
})
</script>

<template>
  <div class="serial-page">
    <!-- 左侧配置区 -->
    <aside class="config-panel">
      <!-- 连接状态 -->
      <div class="status-section">
        <div class="status-indicator" :class="{ connected: isConnected }">
          <div class="status-dot"></div>
          <span class="status-text">{{ isConnected ? t('serial.connected') : t('serial.disconnected') }}</span>
        </div>
        <div v-if="isConnected && currentConnection" class="connection-info">
          <span>{{ currentConnection.config.port_name }}</span>
          <span class="baud">{{ currentConnection.config.baud_rate }} bps</span>
        </div>
      </div>

      <NDivider style="margin: 16px 0" />

      <!-- 串口配置 -->
      <div class="config-section">
        <div class="section-title">
          <NIcon :component="SettingsOutline" size="16" />
          <span>{{ t('serial.config') }}</span>
        </div>

        <div class="config-item">
          <label>{{ t('serial.port') }}</label>
          <NInputGroup>
            <NSelect
              v-model:value="portName"
              :options="portOptions"
              :disabled="isConnected"
              :virtual-scroll="false"
              :placeholder="t('serial.selectPort')"
              size="small"
              style="flex: 1"
              @update:show="onPortSelectShow"
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
            v-model:value="baudRate"
            :options="baudRateOptions"
            :disabled="isConnected"
            size="small"
          />
        </div>

        <div class="config-row">
          <div class="config-item half">
            <label>{{ t('serial.dataBits') }}</label>
            <NSelect
              v-model:value="dataBits"
              :options="dataBitsOptions"
              :disabled="isConnected"
              size="small"
            />
          </div>
          <div class="config-item half">
            <label>{{ t('serial.stopBits') }}</label>
            <NSelect
              v-model:value="stopBits"
              :options="stopBitsOptions"
              :disabled="isConnected"
              size="small"
            />
          </div>
        </div>

        <div class="config-item">
          <label>{{ t('serial.parity') }}</label>
          <NSelect
            v-model:value="parity"
            :options="parityOptions"
            :disabled="isConnected"
            size="small"
          />
        </div>
      </div>

      <!-- 连接按钮 -->
      <NButton
        :type="isConnected ? 'error' : 'primary'"
        :loading="loading"
        :disabled="!portName && !isConnected"
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

      <!-- 统计 -->
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
      <!-- 终端区域 -->
      <div class="terminal-section">
        <div class="terminal-header">
          <div class="terminal-title">
            <NIcon :component="SwapHorizontalOutline" size="18" />
            <span>{{ t('serial.terminal') }}</span>
            <NTag size="small" :bordered="false" type="info">{{ receivedData.length }}</NTag>
          </div>
          <NSpace align="center" :size="8">
            <NSelect
              v-model:value="encoding"
              :options="encodingOptions"
              size="small"
              style="width: 90px"
            />
            <NSwitch v-model:value="hexDisplay" size="small">
              <template #checked>HEX</template>
              <template #unchecked>{{ t('serial.text') }}</template>
            </NSwitch>
            <NSwitch v-model:value="autoScroll" size="small">
              <template #checked>{{ t('serial.scroll') }}</template>
              <template #unchecked>{{ t('serial.scroll') }}</template>
            </NSwitch>
            <NButton size="small" quaternary @click="clearLog">
              <template #icon><NIcon :component="TrashOutline" /></template>
              {{ t('serial.clear') }}
            </NButton>
          </NSpace>
        </div>

        <NScrollbar ref="scrollbarRef" class="terminal-content">
          <div class="terminal-body">
            <div 
              v-for="(item, idx) in receivedData" 
              :key="idx"
              class="log-line"
              :class="item.type"
            >
              <span class="log-time">{{ item.time }}</span>
              <span class="log-type">
                {{ item.type === 'tx' ? 'TX' : item.type === 'rx' ? 'RX' : item.type === 'system' ? 'SYS' : 'ERR' }}
              </span>
              <span class="log-content">
                {{ hexDisplay && item.rawBytes ? formatHex(item.rawBytes) : item.content }}
              </span>
            </div>
            <div v-if="receivedData.length === 0" class="terminal-empty">
              <NIcon :component="SwapHorizontalOutline" size="40" />
              <p>{{ t('serial.emptyHint') }}</p>
            </div>
          </div>
        </NScrollbar>
      </div>

      <!-- 发送区域 -->
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
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.serial-page {
  display: flex;
  height: 100%;
  background: var(--bg-page);
  gap: 16px;
  padding: 16px;
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
