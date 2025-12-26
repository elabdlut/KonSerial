<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  NButton, NSelect, NSpace, NInput, NTag, NIcon, NTooltip, NSwitch,
  NScrollbar, NDivider, NStatistic, NInputGroup,
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
  type SerialPortConfig,
} from '@/stores/serial'

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
const receivedData = ref<{ type: string; content: string; time: string }[]>([])

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

const portOptions = computed(() => 
  availablePorts.value.map(p => ({ 
    label: getPortDisplayName(p), 
    value: p.port_name 
  }))
)

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

const dataBitsOptions = [
  { label: '5 位', value: 5 },
  { label: '6 位', value: 6 },
  { label: '7 位', value: 7 },
  { label: '8 位', value: 8 },
]

const stopBitsOptions = [
  { label: '1 位', value: 1 },
  { label: '2 位', value: 2 },
]

const parityOptions = [
  { label: '无校验', value: 'None' },
  { label: '奇校验', value: 'Odd' },
  { label: '偶校验', value: 'Even' },
]

// 刷新串口
const refreshPorts = async () => {
  loading.value = true
  try {
    await doRefreshPorts()
    if (availablePorts.value.length > 0 && !portName.value) {
      portName.value = availablePorts.value[0].port_name
    }
    message.success(`发现 ${availablePorts.value.length} 个串口`)
  } catch (e) {
    message.error(`刷新失败: ${e}`)
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
      addLog('system', '连接已断开')
      message.info('串口已断开')
    } else {
      if (!portName.value) {
        message.warning('请先选择串口')
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
      addLog('system', `已连接 ${portName.value} (${baudRate.value} bps, ${dataBits.value}-${parity.value[0]}-${stopBits.value})`)
      message.success('串口连接成功')
    }
  } catch (e) {
    addLog('error', `${e}`)
    message.error(`操作失败: ${e}`)
  } finally {
    loading.value = false
  }
}

// 发送
const handleSend = async () => {
  if (!sendText.value.trim() || !currentConnectionId.value) return
  try {
    const bytes = await doSendData(currentConnectionId.value, sendText.value, hexSend.value)
    addLog('tx', sendText.value)
    sendText.value = ''
  } catch (e) {
    message.error(`发送失败: ${e}`)
  }
}

const addLog = (type: string, content: string) => {
  const now = new Date()
  const time = now.toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })
  receivedData.value.push({ type, content, time })
  
  // 如果是接收的数据，同步到全局缓存供波形图使用
  if (type === 'rx') {
    addReceivedData(content)
  }
}

const clearLog = () => {
  receivedData.value = []
}

onMounted(async () => {
  await updateGlobalInfo()
  await refreshPorts()
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
          <span class="status-text">{{ isConnected ? '已连接' : '未连接' }}</span>
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
          <span>串口配置</span>
        </div>

        <div class="config-item">
          <label>串口</label>
          <NInputGroup>
            <NSelect
              v-model:value="portName"
              :options="portOptions"
              :disabled="isConnected"
              placeholder="选择..."
              size="small"
              style="flex: 1"
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
              刷新串口
            </NTooltip>
          </NInputGroup>
        </div>

        <div class="config-item">
          <label>波特率</label>
          <NSelect
            v-model:value="baudRate"
            :options="baudRateOptions"
            :disabled="isConnected"
            size="small"
          />
        </div>

        <div class="config-row">
          <div class="config-item half">
            <label>数据位</label>
            <NSelect
              v-model:value="dataBits"
              :options="dataBitsOptions"
              :disabled="isConnected"
              size="small"
            />
          </div>
          <div class="config-item half">
            <label>停止位</label>
            <NSelect
              v-model:value="stopBits"
              :options="stopBitsOptions"
              :disabled="isConnected"
              size="small"
            />
          </div>
        </div>

        <div class="config-item">
          <label>校验</label>
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
        {{ isConnected ? '断开连接' : '打开连接' }}
      </NButton>

      <!-- 统计 -->
      <div v-if="isConnected" class="stats-section">
        <NDivider style="margin: 20px 0 16px" />
        <div class="section-title">
          <NIcon :component="PulseOutline" size="16" />
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
      <!-- 终端区域 -->
      <div class="terminal-section">
        <div class="terminal-header">
          <div class="terminal-title">
            <NIcon :component="SwapHorizontalOutline" size="18" />
            <span>数据终端</span>
            <NTag size="small" :bordered="false" type="info">{{ receivedData.length }}</NTag>
          </div>
          <NSpace>
            <NSwitch v-model:value="hexDisplay" size="small">
              <template #checked>HEX</template>
              <template #unchecked>ASCII</template>
            </NSwitch>
            <NButton size="small" quaternary @click="clearLog">
              <template #icon><NIcon :component="TrashOutline" /></template>
              清空
            </NButton>
          </NSpace>
        </div>

        <NScrollbar class="terminal-content">
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
              <span class="log-content">{{ item.content }}</span>
            </div>
            <div v-if="receivedData.length === 0" class="terminal-empty">
              <NIcon :component="SwapHorizontalOutline" size="40" />
              <p>连接串口后开始通信</p>
            </div>
          </div>
        </NScrollbar>
      </div>

      <!-- 发送区域 -->
      <div class="send-section">
        <div class="send-options">
          <NSwitch v-model:value="hexSend" size="small">
            <template #checked>HEX发送</template>
            <template #unchecked>文本发送</template>
          </NSwitch>
        </div>
        <div class="send-input">
          <NInput
            v-model:value="sendText"
            :disabled="!isConnected"
            :placeholder="hexSend ? '输入十六进制数据，如: 01 02 03 FF' : '输入要发送的文本...'"
            @keydown.enter="handleSend"
            clearable
          />
          <NButton
            type="primary"
            :disabled="!isConnected || !sendText.trim()"
            @click="handleSend"
          >
            <template #icon><NIcon :component="SendOutline" /></template>
            发送
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
  background: #f5f7fa;
  gap: 16px;
  padding: 16px;
}

/* 左侧配置面板 */
.config-panel {
  width: 280px;
  flex-shrink: 0;
  background: #fff;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.08);
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
  background: #f5f5f5;
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
  font-size: 14px;
  font-weight: 500;
  color: #666;
}

.status-indicator.connected .status-text {
  color: #2e7d32;
}

.connection-info {
  margin-top: 12px;
  font-size: 13px;
  color: #666;
  display: flex;
  justify-content: center;
  gap: 8px;
}

.connection-info .baud {
  color: #999;
}

/* 配置区 */
.config-section {
  flex: 1;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 600;
  color: #333;
  margin-bottom: 16px;
}

.config-item {
  margin-bottom: 12px;
}

.config-item label {
  display: block;
  font-size: 12px;
  color: #666;
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
  background: #f8f9fa;
  border-radius: 8px;
  padding: 12px;
  text-align: center;
}

.stat-label {
  display: block;
  font-size: 11px;
  color: #999;
  margin-bottom: 4px;
}

.stat-value {
  font-size: 18px;
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
  background: #1e1e1e;
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
  font-size: 14px;
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
  font-size: 13px;
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
  font-size: 14px;
}

/* 发送区域 */
.send-section {
  background: #fff;
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.08);
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
