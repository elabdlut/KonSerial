<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, onActivated, onDeactivated, nextTick } from 'vue'
import {
  NButton, NSpace, NIcon, NSwitch, NSlider, NInputNumber, NSelect,
  NTooltip, NDivider, NTag, NCheckbox, NCheckboxGroup,
  useMessage
} from 'naive-ui'
import {
  PlayOutline, PauseOutline, TrashOutline, SettingsOutline,
  AnalyticsOutline, DownloadOutline, HelpCircleOutline,
  CameraOutline
} from '@vicons/ionicons5'
import { receivedBuffer, currentConnectionId, activeConnections } from '@/stores/serial'
import { activeConnections as networkActiveConnections } from '@/stores/network'
import { t } from '@/stores/i18n'

const message = useMessage()

// 图表状态
const isRunning = ref(false)
const showFormatHelp = ref(false)
const canvasRef = ref<HTMLCanvasElement | null>(null)

// 选择要监控的连接
const chartConnectionId = ref<string | null>(currentConnectionId.value)

const connectionOptions = computed(() => [
  ...activeConnections.value.map(c => ({
    label: `${c.config.port_name} (Serial)`,
    value: c.connection_id,
  })),
  ...networkActiveConnections.value.map(c => ({
    label: `${c.config.protocol.toUpperCase()} ${c.config.host}:${c.config.port} (Network)`,
    value: c.connection_id,
  })),
])

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

// ========== 视图缩放/平移 ==========
const zoomX = ref(1)           // 时间轴缩放 (>1 = 放大看更细节)
const panTimeMs = ref(0)       // 时间轴偏移 (ms, >0 = 查看过去数据)
const isLiveMode = ref(true)   // 是否跟随最新数据
let frozenTime: number | null = null  // 暂停时冻结的参考时间
let isDragging = false
let lastDragX = 0
let lastDragY = 0
let pendingRedraw: number | null = null
// 当前帧绘图参数 (供事件处理器将像素坐标转换为数据坐标)
let plotInfo = { x: 56, y: 20, w: 0, h: 0, tMin: 0, tMax: 0, yMin: 0, yMax: 0 }

// 通道颜色
const channelColors = ['#1976d2', '#d32f2f', '#388e3c', '#7b1fa2', '#f57c00', '#0097a7', '#e91e63', '#00bcd4']
const getChannelColor = (idx: number) => channelColors[idx % channelColors.length]

// 数据解析跟踪
let lastProcessedIndex = 0
let processingInterval: number | null = null
let animFrameId: number | null = null

// 总数据点数
const totalDataPoints = computed(() => {
  return Object.values(channelsData.value).reduce((sum, data) => sum + data.length, 0)
})

// 获取通道统计信息
const getChannelStats = (channelName: string) => {
  const data = channelsData.value[channelName]
  if (!data || data.length === 0) {
    return { current: '--', avg: '--', min: '--', max: '--' }
  }
  const values = data.map(d => d.value)
  const avg = values.reduce((a, b) => a + b, 0) / values.length
  const current = values[values.length - 1]
  const min = Math.min(...values.slice(-200))
  const max = Math.max(...values.slice(-200))
  return {
    current: current.toFixed(2),
    avg: avg.toFixed(2),
    min: min.toFixed(2),
    max: max.toFixed(2),
  }
}

// 解析单行数据: 支持 "name:value" 和 "name:v1,v2,v3" 格式
const parseLine = (line: string, time: number) => {
  const trimmed = line.trim()
  // 匹配 name:values 格式
  const colonIdx = trimmed.indexOf(':')
  if (colonIdx <= 0) return

  const channelBase = trimmed.substring(0, colonIdx)
  const valuePart = trimmed.substring(colonIdx + 1)

  // 支持逗号分隔的多值: name:1.2,3.4,5.6 → name_0, name_1, name_2
  const parts = valuePart.split(',')
  parts.forEach((part, idx) => {
    const value = parseFloat(part.trim())
    if (isNaN(value)) return

    const channelName = parts.length > 1 ? `${channelBase}_${idx}` : channelBase

    if (!discoveredChannels.value.includes(channelName)) {
      discoveredChannels.value.push(channelName)
      selectedChannels.value.push(channelName)
    }

    if (!channelsData.value[channelName]) {
      channelsData.value[channelName] = []
    }
    channelsData.value[channelName].push({ time, value })

    const maxPoints = timeRange.value * 100
    if (channelsData.value[channelName].length > maxPoints) {
      channelsData.value[channelName].shift()
    }
  })
}

// 处理接收缓存中的新数据
const processNewData = () => {
  const buffer = receivedBuffer.value
  // 如果缓冲区被裁剪导致索引越界，重置索引到最后位置
  if (lastProcessedIndex > buffer.length) {
    lastProcessedIndex = Math.max(0, buffer.length - 1)
  }
  while (lastProcessedIndex < buffer.length) {
    const item = buffer[lastProcessedIndex]
    if (chartConnectionId.value && item.connection_id !== chartConnectionId.value) {
      lastProcessedIndex++
      continue
    }
    const lines = item.content.split('\n')
    for (const line of lines) {
      if (line.trim()) {
        parseLine(line, item.time)
      }
    }
    lastProcessedIndex++
  }
}

// ========== Canvas 渲染 ==========

const drawChart = () => {
  const canvas = canvasRef.value
  if (!canvas) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  // 适配 DPR
  const dpr = window.devicePixelRatio || 1
  const rect = canvas.getBoundingClientRect()
  canvas.width = rect.width * dpr
  canvas.height = rect.height * dpr
  ctx.scale(dpr, dpr)

  const W = rect.width
  const H = rect.height
  const pad = { top: 20, right: 20, bottom: 36, left: 56 }
  const plotW = W - pad.left - pad.right
  const plotH = H - pad.top - pad.bottom

  // 清空
  ctx.clearRect(0, 0, W, H)

  // 计算时间范围
  const refTime = frozenTime ?? Date.now()
  const visibleSpan = (timeRange.value * 1000) / zoomX.value
  let tMax: number
  if (isLiveMode.value) {
    tMax = refTime
  } else {
    tMax = refTime - panTimeMs.value
  }
  const tMin = tMax - visibleSpan

  // 计算 Y 范围
  let computedYMin = yMin.value
  let computedYMax = yMax.value
  if (autoScale.value) {
    let allMin = Infinity, allMax = -Infinity
    for (const ch of selectedChannels.value) {
      const data = channelsData.value[ch]
      if (!data) continue
      for (const d of data) {
        if (d.time >= tMin) {
          if (d.value < allMin) allMin = d.value
          if (d.value > allMax) allMax = d.value
        }
      }
    }
    if (allMin === Infinity) { allMin = 0; allMax = 100 }
    const margin = (allMax - allMin) * 0.1 || 5
    computedYMin = allMin - margin
    computedYMax = allMax + margin
  }
  const yRange = computedYMax - computedYMin || 1

  // 存储当前帧参数供交互事件使用
  plotInfo = { x: pad.left, y: pad.top, w: plotW, h: plotH, tMin, tMax, yMin: computedYMin, yMax: computedYMax }

  // 绘制背景
  const bgColor = getComputedStyle(canvas).getPropertyValue('--bg-card').trim() || '#ffffff'
  ctx.fillStyle = bgColor
  ctx.fillRect(0, 0, W, H)

  // 从 CSS 变量读取图表配色
  const chartGrid = getComputedStyle(canvas).getPropertyValue('--chart-grid').trim() || '#e8e8e8'
  const chartAxis = getComputedStyle(canvas).getPropertyValue('--chart-axis').trim() || '#ccc'
  const chartLabel = getComputedStyle(canvas).getPropertyValue('--chart-label').trim() || '#999'

  // 绘制网格
  if (gridEnabled.value) {
    ctx.strokeStyle = chartGrid
    ctx.lineWidth = 0.5
    // 水平网格线 (5 lines)
    for (let i = 0; i <= 5; i++) {
      const y = pad.top + (plotH / 5) * i
      ctx.beginPath()
      ctx.moveTo(pad.left, y)
      ctx.lineTo(pad.left + plotW, y)
      ctx.stroke()
    }
    // 垂直网格线 (6 lines)
    for (let i = 0; i <= 6; i++) {
      const x = pad.left + (plotW / 6) * i
      ctx.beginPath()
      ctx.moveTo(x, pad.top)
      ctx.lineTo(x, pad.top + plotH)
      ctx.stroke()
    }
  }

  // 绘制坐标轴
  ctx.strokeStyle = chartAxis
  ctx.lineWidth = 1
  ctx.beginPath()
  ctx.moveTo(pad.left, pad.top)
  ctx.lineTo(pad.left, pad.top + plotH)
  ctx.lineTo(pad.left + plotW, pad.top + plotH)
  ctx.stroke()

  // Y 轴标签
  ctx.fillStyle = chartLabel
  ctx.font = '11px SF Mono, Monaco, monospace'
  ctx.textAlign = 'right'
  ctx.textBaseline = 'middle'
  for (let i = 0; i <= 5; i++) {
    const val = computedYMax - (yRange / 5) * i
    const y = pad.top + (plotH / 5) * i
    ctx.fillText(val.toFixed(1), pad.left - 6, y)
  }

  // X 轴标签 (时间)
  ctx.textAlign = 'center'
  ctx.textBaseline = 'top'
  for (let i = 0; i <= 6; i++) {
    const t = tMin + ((tMax - tMin) / 6) * i
    const x = pad.left + (plotW / 6) * i
    const date = new Date(t)
    const label = `${date.getMinutes().toString().padStart(2, '0')}:${date.getSeconds().toString().padStart(2, '0')}`
    ctx.fillText(label, x, pad.top + plotH + 8)
  }

  // 绘制每个选中通道的曲线
  for (const ch of selectedChannels.value) {
    const data = channelsData.value[ch]
    if (!data || data.length === 0) continue

    const chIdx = discoveredChannels.value.indexOf(ch)
    ctx.strokeStyle = getChannelColor(chIdx)
    ctx.lineWidth = lineWidth.value
    ctx.lineJoin = 'round'
    ctx.beginPath()

    let started = false
    for (const d of data) {
      if (d.time < tMin) continue
      const x = pad.left + ((d.time - tMin) / (tMax - tMin)) * plotW
      const y = pad.top + plotH - ((d.value - computedYMin) / yRange) * plotH
      if (!started) {
        ctx.moveTo(x, y)
        started = true
      } else {
        ctx.lineTo(x, y)
      }
    }
    ctx.stroke()
  }

  // 如果正在运行，继续绘制
  if (isRunning.value) {
    animFrameId = requestAnimationFrame(drawChart)
  }
}

const startAnimation = () => {
  if (animFrameId !== null) return
  animFrameId = requestAnimationFrame(drawChart)
}

const stopAnimation = () => {
  if (animFrameId !== null) {
    cancelAnimationFrame(animFrameId)
    animFrameId = null
  }
}

// ========== 视图重置 ==========

const snapToLive = () => {
  zoomX.value = 1
  panTimeMs.value = 0
  isLiveMode.value = true
  autoScale.value = true
}

const triggerRedraw = () => {
  if (isRunning.value) return // rAF 循环自动处理
  if (pendingRedraw !== null) return
  pendingRedraw = requestAnimationFrame(() => {
    pendingRedraw = null
    drawChart()
  })
}

// 开始/停止采集
const toggleRunning = () => {
  if (!isRunning.value && !chartConnectionId.value) {
    message.warning(t('chart.noConnection'))
    return
  }
  isRunning.value = !isRunning.value
  if (isRunning.value) {
    frozenTime = null
    snapToLive()
    lastProcessedIndex = receivedBuffer.value.length
    processingInterval = window.setInterval(processNewData, 50)
    startAnimation()
    message.success(t('chart.startedMsg'))
  } else {
    frozenTime = Date.now()
    if (processingInterval !== null) {
      clearInterval(processingInterval)
      processingInterval = null
    }
    stopAnimation()
    drawChart() // 暂停后绘制最后一帧
    message.info(t('chart.pausedMsg'))
  }
}

const clearChart = () => {
  channelsData.value = {}
  discoveredChannels.value = []
  selectedChannels.value = []
  lastProcessedIndex = 0
  snapToLive()
  frozenTime = null
  // 重绘空图表
  nextTick(drawChart)
  message.success(t('chart.cleared'))
}

// 导出数据为 CSV
const exportData = () => {
  if (totalDataPoints.value === 0) {
    message.warning(t('chart.noExportData'))
    return
  }

  let csv = 'time,' + discoveredChannels.value.join(',') + '\n'
  const allTimes = new Set<number>()
  for (const ch of discoveredChannels.value) {
    for (const d of channelsData.value[ch] || []) {
      allTimes.add(d.time)
    }
  }

  const sortedTimes = Array.from(allTimes).sort((a, b) => a - b)
  for (const time of sortedTimes) {
    const row = [time.toString()]
    for (const ch of discoveredChannels.value) {
      const data = channelsData.value[ch]?.find(d => d.time === time)
      row.push(data ? data.value.toString() : '')
    }
    csv += row.join(',') + '\n'
  }

  let url: string | null = null
  try {
    const blob = new Blob([csv], { type: 'text/csv' })
    url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `chart_data_${Date.now()}.csv`
    a.click()
    message.success(t('chart.exported'))
  } finally {
    if (url) URL.revokeObjectURL(url)
  }
}

// 导出图表为 PNG — 直接从 canvas 导出
const exportChart = () => {
  const canvas = canvasRef.value
  if (!canvas) {
    message.warning(t('chart.areaNotFound'))
    return
  }
  // 先绘制一帧确保最新
  drawChart()

  try {
    const link = document.createElement('a')
    link.download = `chart_${Date.now()}.png`
    link.href = canvas.toDataURL('image/png')
    link.click()
    message.success(t('chart.savedPng'))
  } catch (err: unknown) {
    message.error(t('chart.exportFail', String(err)))
  }
}

// 窗口 resize 重绘
const onResize = () => {
  if (!isRunning.value) {
    drawChart()
  }
}

// ========== 交互事件: 缩放 & 平移 ==========

const handleWheel = (e: WheelEvent) => {
  e.preventDefault()
  if (plotInfo.w <= 0 || plotInfo.h <= 0) return
  const canvas = canvasRef.value
  if (!canvas) return

  const rect = canvas.getBoundingClientRect()
  const mx = e.clientX - rect.left
  const my = e.clientY - rect.top
  const ratioX = Math.max(0, Math.min(1, (mx - plotInfo.x) / plotInfo.w))
  const ratioY = Math.max(0, Math.min(1, (my - plotInfo.y) / plotInfo.h))

  // 缩放因子 (deltaY < 0 = 向上/pinch放大 → zoom in)
  const zf = e.deltaY < 0 ? 1.12 : 1 / 1.12

  // 时间轴缩放 (cursor-centered)
  {
    const oldSpan = plotInfo.tMax - plotInfo.tMin
    const newSpan = oldSpan / zf
    const cursorT = plotInfo.tMin + ratioX * oldSpan
    const newTMax = cursorT + (1 - ratioX) * newSpan

    const newZoomX = (timeRange.value * 1000) / newSpan
    zoomX.value = Math.max(0.05, Math.min(200, newZoomX))

    const refTime = frozenTime ?? Date.now()
    panTimeMs.value = Math.max(0, refTime - newTMax)
    isLiveMode.value = panTimeMs.value < 50
  }

  // Ctrl + wheel / 触摸板 pinch → 同时缩放 Y 轴
  if (e.ctrlKey) {
    const oldYRange = plotInfo.yMax - plotInfo.yMin
    const newYRange = oldYRange / zf
    const cursorY = plotInfo.yMax - ratioY * oldYRange
    const newYMax = cursorY + ratioY * newYRange

    autoScale.value = false
    yMax.value = newYMax
    yMin.value = newYMax - newYRange
  }

  triggerRedraw()
}

const handleMouseDown = (e: MouseEvent) => {
  if (e.button !== 0) return
  isDragging = true
  lastDragX = e.clientX
  lastDragY = e.clientY
  const canvas = canvasRef.value
  if (canvas) canvas.style.cursor = 'grabbing'
}

const handleMouseMove = (e: MouseEvent) => {
  if (!isDragging) return

  const dx = e.clientX - lastDragX
  const dy = e.clientY - lastDragY
  lastDragX = e.clientX
  lastDragY = e.clientY
  if (plotInfo.w <= 0 || plotInfo.h <= 0) return

  // 时间轴平移: 向右拖 → 看过去
  const timeSpan = plotInfo.tMax - plotInfo.tMin
  const dtMs = -(dx / plotInfo.w) * timeSpan
  panTimeMs.value = Math.max(0, panTimeMs.value + dtMs)
  isLiveMode.value = panTimeMs.value < 50

  // Y 轴平移: 向上拖 → 看更大值
  if (autoScale.value) {
    autoScale.value = false
    yMin.value = plotInfo.yMin
    yMax.value = plotInfo.yMax
  }
  const yRange = plotInfo.yMax - plotInfo.yMin
  const dyVal = (dy / plotInfo.h) * yRange
  yMin.value += dyVal
  yMax.value += dyVal

  triggerRedraw()
}

const handleMouseUp = () => {
  if (!isDragging) return
  isDragging = false
  const canvas = canvasRef.value
  if (canvas) canvas.style.cursor = 'grab'
}

const handleDblClick = () => {
  snapToLive()
  frozenTime = isRunning.value ? null : frozenTime
  triggerRedraw()
}

// 配置变化时重绘
watch([gridEnabled, lineWidth, autoScale, yMin, yMax, selectedChannels, zoomX, panTimeMs], () => {
  if (!isRunning.value && totalDataPoints.value > 0) {
    nextTick(drawChart)
  }
})

// 切换连接时清空旧数据
watch(chartConnectionId, () => {
  clearChart()
})

const cleanupChart = () => {
  window.removeEventListener('resize', onResize)
  window.removeEventListener('mousemove', handleMouseMove)
  window.removeEventListener('mouseup', handleMouseUp)
  if (pendingRedraw !== null) {
    cancelAnimationFrame(pendingRedraw)
    pendingRedraw = null
  }
  stopAnimation()
  if (processingInterval !== null) {
    clearInterval(processingInterval)
    processingInterval = null
  }
}

const initChart = () => {
  window.addEventListener('resize', onResize)
  window.addEventListener('mousemove', handleMouseMove)
  window.addEventListener('mouseup', handleMouseUp)
  nextTick(drawChart)
  // 如果之前在运行状态，恢复动画和数据处理
  if (isRunning.value) {
    if (processingInterval === null) {
      processingInterval = window.setInterval(processNewData, 50)
    }
    startAnimation()
  }
}

onMounted(initChart)

onUnmounted(cleanupChart)

// keep-alive 生命周期：切换标签页时暂停/恢复，避免资源泄漏
onActivated(initChart)
onDeactivated(cleanupChart)
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

      <!-- 连接选择 -->
      <div class="config-section">
        <div class="section-title">
          <NIcon :component="AnalyticsOutline" size="16" />
          <span>{{ t('chart.targetConnection') }}</span>
        </div>
        <NSelect
          v-model:value="chartConnectionId"
          :options="connectionOptions"
          :placeholder="t('chart.selectConnection')"
          size="small"
          clearable
        />
      </div>

      <NDivider style="margin: 12px 0" />

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
          <code>name:v1,v2,v3</code>
          <p class="examples">{{ t('chart.examples') }}</p>
          <code>temp:25.5</code>
          <code>imu:1.2,3.4,5.6</code>
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
      <div class="chart-container">
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
          </NSpace>
        </div>

        <div class="chart-body">
          <canvas
            ref="canvasRef"
            class="chart-canvas"
            @wheel.prevent="handleWheel"
            @mousedown="handleMouseDown"
            @dblclick="handleDblClick"
          ></canvas>
          <div v-if="!isLiveMode && totalDataPoints > 0" class="go-live-btn" @click="handleDblClick">
            {{ t('chart.goLive') }}
          </div>
          <div v-if="totalDataPoints === 0 && !isRunning" class="chart-empty-overlay">
            <NIcon :component="AnalyticsOutline" size="48" />
            <p>{{ t('chart.noData') }}</p>
            <p class="sub">{{ t('chart.startHint') }}</p>
            <div class="format-example">
              <p>{{ t('chart.dataFormat') }} <code>name:value</code></p>
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

.status-section { text-align: center; }

.status-indicator {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: var(--bg-page);
  border-radius: 20px;
  transition: all 0.3s;
}
.status-indicator.running { background: var(--chart-running-bg); }

.status-dot {
  width: 8px; height: 8px;
  border-radius: 50%;
  background: #bbb;
}
.status-indicator.running .status-dot {
  background: var(--chart-running-dot);
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
.status-indicator.running .status-text { color: #1565c0; }

.data-count {
  margin-top: 8px;
  font-size: var(--font-xs);
  color: var(--text-muted);
}

.config-section { margin-bottom: 8px; }

.section-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-sm);
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 12px;
}

.config-item { margin-bottom: 12px; }
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
.config-item.row label { margin-bottom: 0; }
.config-row { display: flex; gap: 12px; }
.config-item.half { flex: 1; }

.slider-value {
  text-align: right;
  font-size: var(--font-xs);
  color: var(--text-secondary);
  margin-top: 4px;
}

.control-buttons { margin-top: 16px; }
.stats-section { margin-top: auto; }

/* 右侧图表区域 */
.main-area { flex: 1; min-width: 0; }

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
  position: relative;
  min-height: 0;
}

.chart-canvas {
  width: 100%;
  height: 100%;
  display: block;
  cursor: grab;
}
.chart-canvas:active {
  cursor: grabbing;
}

.go-live-btn {
  position: absolute;
  bottom: 16px;
  right: 16px;
  background: var(--chart-running-dot);
  color: white;
  padding: 6px 16px;
  border-radius: 20px;
  font-size: var(--font-xs);
  cursor: pointer;
  z-index: 10;
  box-shadow: 0 2px 8px rgba(25, 118, 210, 0.4);
  transition: all 0.2s;
  user-select: none;
}
.go-live-btn:hover {
  background: #1565c0;
  box-shadow: 0 4px 12px rgba(25, 118, 210, 0.5);
}

.chart-empty-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  pointer-events: none;
}

.chart-empty-overlay p {
  margin-top: 12px;
  font-size: var(--font-base);
}
.chart-empty-overlay .sub {
  font-size: var(--font-xs);
  color: #bbb;
  margin-top: 4px;
}

.format-example {
  margin-top: 16px;
  padding: 12px 16px;
  background: var(--bg-page);
  border-radius: 8px;
  pointer-events: auto;
}
.format-example p {
  margin: 0;
  font-size: var(--font-xs);
  color: var(--text-muted);
}
.format-example code {
  background: var(--bg-card);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: 'SF Mono', Monaco, monospace;
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
.hint-content p { margin: 6px 0 4px; color: #888; }
.hint-content p.examples { margin-top: 10px; }
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
.channel-list { max-height: 120px; overflow-y: auto; }
.channel-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
}
.channel-color { width: 4px; height: 16px; border-radius: 2px; }

/* 通道统计 */
.empty-stats {
  text-align: center;
  padding: 16px;
  color: #999;
  font-size: var(--font-xs);
}
.channel-stats { display: flex; flex-direction: column; gap: 8px; }
.channel-stat-card {
  background: var(--bg-page);
  border-radius: 8px;
  padding: 10px;
}
.channel-stat-header {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--font-xs);
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 8px;
}
.channel-color-dot { width: 8px; height: 8px; border-radius: 50%; }
.stats-mini-grid { display: flex; gap: 12px; }
.stat-mini { flex: 1; }
.stat-mini .label {
  display: block;
  font-size: var(--font-2xs);
  color: var(--text-muted);
}
.stat-mini .value {
  font-size: var(--font-base);
  font-weight: 600;
  font-family: 'SF Mono', Monaco, monospace;
  color: var(--text-primary);
}
</style>
