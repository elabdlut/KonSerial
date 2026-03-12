<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import {
  NButton, NSpace, NIcon, NSwitch, NSlider, NInputNumber,
  NTooltip, NDivider, NTag, NCheckbox, NCheckboxGroup,
  useMessage
} from 'naive-ui'
import {
  PlayOutline, PauseOutline, TrashOutline, SettingsOutline,
  AnalyticsOutline, DownloadOutline, ExpandOutline, HelpCircleOutline,
  CameraOutline
} from '@vicons/ionicons5'
import { receivedBuffer } from '@/stores/serial'
import { t } from '@/stores/i18n'

const message = useMessage()

// 图表状态
const isRunning = ref(false)
const showFormatHelp = ref(false)
const chartRef = ref<HTMLDivElement | null>(null)

// 多通道数据存储
interface ChannelData {
  time: number
  value: number
}
const channelsData = ref<Record<string, ChannelData[]>>({})

// 已发现的通道列表
const discoveredChannels = ref<string[]>([])
// 选中显示的通道
const selectedChannels = ref<string[]>([])

// 图表配置
const timeRange = ref(30)
const autoScale = ref(true)
const yMin = ref(0)
const yMax = ref(100)
const gridEnabled = ref(true)
const lineWidth = ref(2)

// 通道颜色
const channelColors = ['#1976d2', '#d32f2f', '#388e3c', '#7b1fa2', '#f57c00', '#0097a7']
const getChannelColor = (idx: number) => channelColors[idx % channelColors.length]

// 数据解析跟踪
let lastProcessedIndex = 0
let processingInterval: number | null = null

// 总数据点数
const totalDataPoints = computed(() => {
  return Object.values(channelsData.value).reduce((sum, data) => sum + data.length, 0)
})

// 获取通道统计信息
const getChannelStats = (channelName: string) => {
  const data = channelsData.value[channelName]
  if (!data || data.length === 0) {
    return { current: '--', avg: '--' }
  }
  const values = data.map(d => d.value)
  const avg = values.reduce((a, b) => a + b, 0) / values.length
  const current = values[values.length - 1]
  return {
    current: current.toFixed(2),
    avg: avg.toFixed(2)
  }
}

// 解析单行数据: "name:value"
const parseLine = (line: string, time: number) => {
  const match = line.trim().match(/^([\w-]+):([-+]?\d*\.?\d+)$/)
  if (match) {
    const channelName = match[1]
    const value = parseFloat(match[2])
    
    if (!isNaN(value)) {
      // 添加新发现的通道
      if (!discoveredChannels.value.includes(channelName)) {
        discoveredChannels.value.push(channelName)
        selectedChannels.value.push(channelName) // 默认选中
      }
      
      // 存储数据
      if (!channelsData.value[channelName]) {
        channelsData.value[channelName] = []
      }
      channelsData.value[channelName].push({ time, value })
      
      // 限制每个通道的数据点数量
      const maxPoints = timeRange.value * 100 // 假设最高100Hz采样
      if (channelsData.value[channelName].length > maxPoints) {
        channelsData.value[channelName].shift()
      }
    }
  }
}

// 处理接收缓存中的新数据
const processNewData = () => {
  const buffer = receivedBuffer.value
  while (lastProcessedIndex < buffer.length) {
    const item = buffer[lastProcessedIndex]
    // 可能一次接收多行
    const lines = item.content.split('\n')
    for (const line of lines) {
      if (line.trim()) {
        parseLine(line, item.time)
      }
    }
    lastProcessedIndex++
  }
}

// 开始/停止采集
const toggleRunning = () => {
  isRunning.value = !isRunning.value
  if (isRunning.value) {
    // 开始定时处理数据
    lastProcessedIndex = receivedBuffer.value.length // 从当前位置开始
    processingInterval = window.setInterval(processNewData, 50) // 50ms更新一次
    message.success(t('chart.startedMsg'))
  } else {
    // 停止定时处理
    if (processingInterval !== null) {
      clearInterval(processingInterval)
      processingInterval = null
    }
    message.info(t('chart.pausedMsg'))
  }
}

const clearChart = () => {
  channelsData.value = {}
  discoveredChannels.value = []
  selectedChannels.value = []
  lastProcessedIndex = 0
  message.success(t('chart.cleared'))
}

// 导出数据为 CSV
const exportData = () => {
  if (totalDataPoints.value === 0) {
    message.warning(t('chart.noExportData'))
    return
  }
  
  // 生成 CSV
  let csv = 'time,' + discoveredChannels.value.join(',') + '\n'
  const allTimes = new Set<number>()
  for (const ch of discoveredChannels.value) {
    for (const d of channelsData.value[ch] || []) {
      allTimes.add(d.time)
    }
  }
  
  const sortedTimes = Array.from(allTimes).sort((a, b) => a - b)
  for (const t of sortedTimes) {
    const row = [t.toString()]
    for (const ch of discoveredChannels.value) {
      const data = channelsData.value[ch]?.find(d => d.time === t)
      row.push(data ? data.value.toString() : '')
    }
    csv += row.join(',') + '\n'
  }
  
  // 下载
  const blob = new Blob([csv], { type: 'text/csv' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `chart_data_${Date.now()}.csv`
  a.click()
  URL.revokeObjectURL(url)
  message.success(t('chart.exported'))
}

// 导出图表为 PNG
const exportChart = async () => {
  const chartEl = chartRef.value
  if (!chartEl) {
    message.warning(t('chart.areaNotFound'))
    return
  }
  
  try {
    const html2canvas = (await import('html2canvas')).default
    const canvas = await html2canvas(chartEl, {
      backgroundColor: '#ffffff',
      scale: 2,
    })
    const link = document.createElement('a')
    link.download = `chart_${Date.now()}.png`
    link.href = canvas.toDataURL('image/png')
    link.click()
    message.success(t('chart.savedPng'))
  } catch (err: unknown) {
    message.error(t('chart.exportFail', String(err)))
  }
}

// 组件卸载时清理
onUnmounted(() => {
  if (processingInterval !== null) {
    clearInterval(processingInterval)
  }
})
</script>

<template>
  <div class="chart-page">
    <!-- 左侧配置区 -->
    <aside class="config-panel">
      <!-- 运行状态 -->
      <div class="status-section">
        <div class="status-indicator" :class="{ running: isRunning }">
          <div class="status-dot"></div>
          <span class="status-text">{{ isRunning ? t('chart.running') : t('chart.stopped') }}</span>
        </div>
        <div class="data-count">
          <span>{{ totalDataPoints }} {{ t('chart.dataPoints') }}</span>
        </div>
      </div>

      <NDivider style="margin: 16px 0" />

      <!-- 数据格式提示 -->
      <div class="format-hint">
        <div class="hint-header" @click="showFormatHelp = !showFormatHelp">
          <NIcon :component="HelpCircleOutline" size="14" />
          <span>{{ t('chart.formatHelp') }}</span>
          <span class="toggle">{{ showFormatHelp ? t('chart.collapse') : t('chart.expand') }}</span>
        </div>
        <div v-show="showFormatHelp" class="hint-content">
          <p>{{ t('chart.formatDesc') }}</p>
          <code>name:value</code>
          <p class="examples">{{ t('chart.examples') }}</p>
          <code>temp:25.5</code>
          <code>voltage:3.3</code>
          <code>sensor:128</code>
          <p class="note">{{ t('chart.formatNote') }}</p>
        </div>
      </div>

      <NDivider style="margin: 12px 0" />

      <!-- 通道选择 -->
      <div class="config-section">
        <div class="section-title">
          <NIcon :component="AnalyticsOutline" size="16" />
          <span>{{ t('chart.channels') }}</span>
          <NTag size="small" :bordered="false">{{ discoveredChannels.length }}</NTag>
        </div>

        <div v-if="discoveredChannels.length === 0" class="empty-channels">
          {{ t('chart.noChannels') }}
        </div>
        <div v-else class="channel-list">
          <NCheckboxGroup v-model:value="selectedChannels">
            <div v-for="(ch, idx) in discoveredChannels" :key="ch" class="channel-item">
              <div class="channel-color" :style="{ background: getChannelColor(idx) }"></div>
              <NCheckbox :value="ch" :label="ch" />
            </div>
          </NCheckboxGroup>
        </div>

        <div class="config-item" style="margin-top: 12px">
          <label>{{ t('chart.timeRange') }}</label>
          <NSlider v-model:value="timeRange" :min="5" :max="120" :step="5" />
          <div class="slider-value">{{ timeRange }}s</div>
        </div>
      </div>

      <NDivider style="margin: 16px 0" />

      <!-- 显示配置 -->
      <div class="config-section">
        <div class="section-title">
          <NIcon :component="SettingsOutline" size="16" />
          <span>{{ t('chart.display') }}</span>
        </div>

        <div class="config-item row">
          <label>{{ t('chart.autoScale') }}</label>
          <NSwitch v-model:value="autoScale" size="small" />
        </div>

        <div v-if="!autoScale" class="config-row">
          <div class="config-item half">
            <label>{{ t('chart.yMin') }}</label>
            <NInputNumber v-model:value="yMin" size="small" :show-button="false" />
          </div>
          <div class="config-item half">
            <label>{{ t('chart.yMax') }}</label>
            <NInputNumber v-model:value="yMax" size="small" :show-button="false" />
          </div>
        </div>

        <div class="config-item row">
          <label>{{ t('chart.showGrid') }}</label>
          <NSwitch v-model:value="gridEnabled" size="small" />
        </div>

        <div class="config-item">
          <label>{{ t('chart.lineWidth') }}</label>
          <NSlider v-model:value="lineWidth" :min="1" :max="5" :step="0.5" />
        </div>
      </div>

      <!-- 控制按钮 -->
      <div class="control-buttons">
        <NButton
          :type="isRunning ? 'error' : 'primary'"
          block
          size="large"
          @click="toggleRunning"
        >
          <template #icon>
            <NIcon :component="isRunning ? PauseOutline : PlayOutline" />
          </template>
          {{ isRunning ? t('chart.pause') : t('chart.start') }}
        </NButton>

        <NSpace style="margin-top: 12px">
          <NButton @click="clearChart" size="small">
            <template #icon><NIcon :component="TrashOutline" /></template>
            {{ t('chart.clear') }}
          </NButton>
          <NButton @click="exportData" size="small">
            <template #icon><NIcon :component="DownloadOutline" /></template>
            {{ t('chart.export') }}
          </NButton>
        </NSpace>
      </div>

      <!-- 统计信息 -->
      <div class="stats-section">
        <NDivider style="margin: 20px 0 16px" />
        <div class="section-title">
          <span>{{ t('chart.channelStats') }}</span>
        </div>
        <div v-if="selectedChannels.length === 0" class="empty-stats">
          {{ t('chart.selectChannels') }}
        </div>
        <div v-else class="channel-stats">
          <div v-for="ch in selectedChannels" :key="ch" class="channel-stat-card">
            <div class="channel-stat-header">
              <div class="channel-color-dot" :style="{ background: getChannelColor(discoveredChannels.indexOf(ch)) }"></div>
              <span>{{ ch }}</span>
            </div>
            <div class="stats-mini-grid">
              <div class="stat-mini">
                <span class="label">{{ t('chart.current') }}</span>
                <span class="value">{{ getChannelStats(ch).current }}</span>
              </div>
              <div class="stat-mini">
                <span class="label">{{ t('chart.average') }}</span>
                <span class="value">{{ getChannelStats(ch).avg }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </aside>

    <!-- 右侧图表区域 -->
    <main class="main-area">
      <div ref="chartRef" class="chart-container">
        <div class="chart-header">
          <div class="chart-title">
            <NIcon :component="AnalyticsOutline" size="18" />
            <span>{{ t('chart.realtime') }}</span>
            <NTag v-if="isRunning" size="small" type="success">{{ t('chart.live') }}</NTag>
          </div>
          <NSpace>
            <NTooltip>
              <template #trigger>
                <NButton size="small" quaternary @click="exportChart">
                  <template #icon><NIcon :component="CameraOutline" /></template>
                </NButton>
              </template>
              {{ t('chart.saveImage') }}
            </NTooltip>
            <NTooltip>
              <template #trigger>
                <NButton size="small" quaternary>
                  <template #icon><NIcon :component="ExpandOutline" /></template>
                </NButton>
              </template>
              {{ t('chart.fullscreen') }}
            </NTooltip>
          </NSpace>
        </div>

        <div class="chart-body">
          <div v-if="totalDataPoints === 0" class="chart-empty">
            <NIcon :component="AnalyticsOutline" size="48" />
            <p>{{ t('chart.noData') }}</p>
            <p class="sub">{{ t('chart.startHint') }}</p>
            <div class="format-example">
              <p>{{ t('chart.dataFormat') }} <code>name:value</code></p>
            </div>
          </div>
          <div v-else class="chart-placeholder">
            <!-- 图表占位符，待集成 ECharts -->
            <div class="mock-chart">
              <div class="y-axis">
                <span>100</span>
                <span>75</span>
                <span>50</span>
                <span>25</span>
                <span>0</span>
              </div>
              <div class="chart-area" :class="{ 'with-grid': gridEnabled }">
                <div class="chart-line"></div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.chart-page {
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
  overflow-y: auto;
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

.status-indicator.running {
  background: #e3f2fd;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #bbb;
}

.status-indicator.running .status-dot {
  background: #1976d2;
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.6; transform: scale(1.2); }
}

.status-text {
  font-size: var(--font-base);
  font-weight: 500;
  color: var(--text-secondary);
}

.status-indicator.running .status-text {
  color: #1565c0;
}

.data-count {
  margin-top: 8px;
  font-size: var(--font-xs);
  color: var(--text-muted);
}

.config-section {
  margin-bottom: 8px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-sm);
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 12px;
}

.config-item {
  margin-bottom: 12px;
}

.config-item.row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.config-item label {
  display: block;
  font-size: var(--font-xs);
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.config-item.row label {
  margin-bottom: 0;
}

.config-row {
  display: flex;
  gap: 12px;
}

.config-item.half {
  flex: 1;
}

.slider-value {
  text-align: right;
  font-size: var(--font-xs);
  color: var(--text-secondary);
  margin-top: 4px;
}

.control-buttons {
  margin-top: 16px;
}

.stats-section {
  margin-top: auto;
}

.stats-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.stat-item {
  background: var(--bg-page);
  border-radius: 8px;
  padding: 10px;
  text-align: center;
}

.stat-label {
  display: block;
  font-size: var(--font-2xs);
  color: var(--text-muted);
  margin-bottom: 2px;
}

.stat-value {
  font-size: var(--font-lg);
  font-weight: 600;
  font-family: 'SF Mono', Monaco, monospace;
}

.stat-value.current { color: #1976d2; }
.stat-value.avg { color: #7b1fa2; }
.stat-value.min { color: #388e3c; }
.stat-value.max { color: #d32f2f; }

/* 右侧图表区域 */
.main-area {
  flex: 1;
  min-width: 0;
}

.chart-container {
  height: 100%;
  background: var(--bg-card);
  border-radius: 12px;
  box-shadow: var(--shadow-card);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.chart-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
}

.chart-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: var(--font-base);
  font-weight: 500;
  color: var(--text-primary);
}

.chart-body {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.chart-empty {
  text-align: center;
  color: var(--text-muted);
}

.chart-empty p {
  margin-top: 12px;
  font-size: var(--font-base);
}

.chart-empty .sub {
  font-size: var(--font-xs);
  color: #bbb;
  margin-top: 4px;
}

.chart-placeholder {
  width: 100%;
  height: 100%;
}

.mock-chart {
  display: flex;
  height: 100%;
}

.y-axis {
  width: 40px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  padding: 10px 0;
  font-size: var(--font-2xs);
  color: var(--text-muted);
  text-align: right;
  padding-right: 8px;
}

.chart-area {
  flex: 1;
  border-left: 1px solid #ddd;
  border-bottom: 1px solid #ddd;
  position: relative;
}

.chart-area.with-grid {
  background-image: 
    linear-gradient(to right, #f0f0f0 1px, transparent 1px),
    linear-gradient(to bottom, #f0f0f0 1px, transparent 1px);
  background-size: 50px 50px;
}

.chart-line {
  position: absolute;
  bottom: 20%;
  left: 0;
  right: 0;
  height: 2px;
  background: #1976d2;
}

/* 数据格式提示 */
.format-hint {
  background: #f0f7ff;
  border-radius: 8px;
  overflow: hidden;
}

.hint-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 12px;
  font-size: var(--font-xs);
  color: #1565c0;
  cursor: pointer;
  user-select: none;
}

.hint-header .toggle {
  margin-left: auto;
  font-size: var(--font-2xs);
  color: #90caf9;
}

.hint-content {
  padding: 0 12px 12px;
  font-size: var(--font-xs);
  color: #555;
}

.hint-content p {
  margin: 6px 0 4px;
  color: #888;
}

.hint-content p.examples {
  margin-top: 10px;
}

.hint-content p.note {
  margin-top: 10px;
  padding: 8px;
  background: #e3f2fd;
  border-radius: 4px;
  color: #1565c0;
  font-size: var(--font-2xs);
}

.hint-content code {
  display: block;
  background: #fff;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  padding: 6px 10px;
  margin: 4px 0;
  font-family: 'SF Mono', Monaco, monospace;
  font-size: var(--font-2xs);
  color: #333;
}

/* 通道列表 */
.empty-channels {
  text-align: center;
  padding: 16px;
  color: #999;
  font-size: var(--font-xs);
}

.channel-list {
  max-height: 120px;
  overflow-y: auto;
}

.channel-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
}

.channel-color {
  width: 4px;
  height: 16px;
  border-radius: 2px;
}

/* 通道统计 */
.empty-stats {
  text-align: center;
  padding: 16px;
  color: #999;
  font-size: var(--font-xs);
}

.channel-stats {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.channel-stat-card {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 10px;
}

.channel-stat-header {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-xs);
  font-weight: 500;
  color: #333;
  margin-bottom: 8px;
}

.channel-color-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.stats-mini-grid {
  display: flex;
  gap: 12px;
}

.stat-mini {
  flex: 1;
}

.stat-mini .label {
  display: block;
  font-size: var(--font-2xs);
  color: #999;
}

.stat-mini .value {
  font-size: var(--font-base);
  font-weight: 600;
  font-family: 'SF Mono', Monaco, monospace;
  color: #333;
}

.format-example {
  margin-top: 16px;
  padding: 12px 16px;
  background: #f5f5f5;
  border-radius: 8px;
}

.format-example p {
  margin: 0;
  font-size: var(--font-xs);
  color: #666;
}

.format-example code {
  background: #fff;
  padding: 2px 6px;
  border-radius: 4px;
  font-family: 'SF Mono', Monaco, monospace;
}
</style>
