// 串口状态管理（多连接架构 + 标签页支持）
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { appConfig } from './config'

// ========== 类型定义 ==========

/** 串口完整配置 */
export interface SerialPortConfig {
  port_name: string
  baud_rate: number
  data_bits: number
  stop_bits: number
  parity: string
  flow_control: string
  timeout_ms: number
  auto_reconnect?: boolean
  reconnect_interval_ms?: number
  max_reconnect_attempts?: number
}

/** 串口状态枚举 */
export type PortStatus =
  | 'Disconnected'
  | 'Connecting'
  | 'Connected'
  | { Error: string }

/** 单个连接的运行时信息 */
export interface ConnectionInfo {
  connection_id: string
  status: PortStatus
  config: SerialPortConfig
  bytes_received: number
  bytes_sent: number
  rx_rate: number
  tx_rate: number
  connected_at: string | null
  last_error: string | null
  created_at: string
}

/** 串口详细信息 */
export interface PortInfo {
  port_name: string
  port_type: string       // "USB", "PCI", "Bluetooth", "Unknown"
  manufacturer: string | null
  product: string | null
  serial_number: string | null
  vid: number | null      // USB Vendor ID
  pid: number | null      // USB Product ID
}

/** 全局运行时信息 */
export interface GlobalRuntimeInfo {
  available_ports: PortInfo[]
  active_connections: ConnectionInfo[]
  total_connections: number
}

/** 串口简单信息（兼容旧版） */
export interface SerialPortInfo {
  port_name: string
  port_type: string
}

/** 终端日志条目 */
export interface LogEntry {
  type: string
  content: string
  time: string
}

/** 终端显示设置 */
export interface DisplaySettings {
  hexDisplay: boolean
  encoding: string
  autoScroll: boolean
}

/** 串口标签页 */
export interface SerialTab {
  id: string
  connectionId: string | null
  name: string
  config: SerialPortConfig
}

// ========== 响应式状态 ==========

/** 全局运行时信息 */
export const globalInfo = ref<GlobalRuntimeInfo | null>(null)

/** 可用串口列表（详细信息） */
export const availablePorts = computed(() => globalInfo.value?.available_ports || [])

/** 获取串口显示名称（包含产品名） */
export function getPortDisplayName(port: PortInfo): string {
  if (port.product) {
    return `${port.port_name} (${port.product})`
  }
  if (port.port_type === 'USB' && port.vid && port.pid) {
    return `${port.port_name} [USB ${port.vid.toString(16).toUpperCase()}:${port.pid.toString(16).toUpperCase()}]`
  }
  if (port.port_type !== 'Unknown') {
    return `${port.port_name} [${port.port_type}]`
  }
  return port.port_name
}

/** 所有活跃连接 */
export const activeConnections = computed(() => globalInfo.value?.active_connections || [])

// 自动重连状态跟踪
const reconnectingIds = new Set<string>()
const reconnectAttemptCounts: Record<string, number> = {}

watch(
  activeConnections,
  (conns) => {
    for (const conn of conns) {
      if (typeof conn.status !== 'string') {
        // 是 Error 对象
        const config = conn.config
        if (config.auto_reconnect && !reconnectingIds.has(conn.connection_id)) {
          const maxAttempts = config.max_reconnect_attempts || 3
          const intervalMs = config.reconnect_interval_ms || 1000
          reconnectAttemptCounts[conn.connection_id] = reconnectAttemptCounts[conn.connection_id] || 0

          if (reconnectAttemptCounts[conn.connection_id] >= maxAttempts) {
            continue
          }

          reconnectingIds.add(conn.connection_id)
          setTimeout(() => {
            reconnectingIds.delete(conn.connection_id)
            const latest = activeConnections.value.find(
              (c) => c.connection_id === conn.connection_id
            )
            if (
              latest &&
              typeof latest.status !== 'string' &&
              latest.config.auto_reconnect
            ) {
              reconnectAttemptCounts[conn.connection_id]++
              const time = new Date().toLocaleTimeString('zh-CN', {
                hour12: false,
                hour: '2-digit',
                minute: '2-digit',
                second: '2-digit',
              })
              addConnectionLog(conn.connection_id, {
                type: 'system',
                content: `自动重连第 ${reconnectAttemptCounts[conn.connection_id]}/${maxAttempts} 次...`,
                time,
              })
              openSerialPort(conn.connection_id, latest.config).catch((e) => {
                const errTime = new Date().toLocaleTimeString('zh-CN', {
                  hour12: false,
                  hour: '2-digit',
                  minute: '2-digit',
                  second: '2-digit',
                })
                addConnectionLog(conn.connection_id, {
                  type: 'error',
                  content: `自动重连失败: ${e}`,
                  time: errTime,
                })
              })
            }
          }, intervalMs)
        }
      } else if (conn.status === 'Connected') {
        // 连接成功时重置计数
        delete reconnectAttemptCounts[conn.connection_id]
      }
    }
  }
)

/** 当前选中的连接 ID（用于图表/脚本等默认目标） */
export const currentConnectionId = ref<string | null>(null)

/** 当前选中的连接信息 */
export const currentConnection = computed(() => {
  if (!currentConnectionId.value) return null
  return activeConnections.value.find(c => c.connection_id === currentConnectionId.value) || null
})

/** 接收数据缓存（全局共享，供波形图等页面使用） */
export interface ReceivedLine {
  connection_id: string
  content: string
  time: number
}
export const receivedBuffer = ref<ReceivedLine[]>([])
import { maxBufferSize as settingsMaxBufferSize } from './settings'
export const maxBufferSize = settingsMaxBufferSize

// ========== 行缓冲状态（解决串口数据分段显示问题） ==========

const connectionByteDecoders: Record<string, TextDecoder> = {}
const connectionPendingTexts: Record<string, string> = {}
const connectionFlushTimers: Record<string, number | null> = {}

// ========== 脚本数据回调（按行触发） ==========

type ScriptLineCallback = (line: string) => void
type AnyScriptLineCallback = (connectionId: string, line: string) => void
const scriptLineCallbacks: Record<string, ScriptLineCallback[]> = {}
const anyScriptLineCallbacks: AnyScriptLineCallback[] = []

/** 注册某个连接的按行数据回调（供脚本使用） */
export function onScriptDataLine(connectionId: string, callback: ScriptLineCallback): () => void {
  if (!scriptLineCallbacks[connectionId]) scriptLineCallbacks[connectionId] = []
  scriptLineCallbacks[connectionId].push(callback)
  return () => {
    const arr = scriptLineCallbacks[connectionId]
    if (!arr) return
    const idx = arr.indexOf(callback)
    if (idx >= 0) arr.splice(idx, 1)
  }
}

/** 注册所有连接的按行数据回调 */
export function onAnyScriptDataLine(callback: AnyScriptLineCallback): () => void {
  anyScriptLineCallbacks.push(callback)
  return () => {
    const idx = anyScriptLineCallbacks.indexOf(callback)
    if (idx >= 0) anyScriptLineCallbacks.splice(idx, 1)
  }
}

function flushConnectionLines(connectionId: string, force = false) {
  let text = connectionPendingTexts[connectionId] || ''
  if (!text) return

  if (!force) {
    const lastIdx = text.lastIndexOf('\n')
    if (lastIdx === -1) return
    connectionPendingTexts[connectionId] = text.slice(lastIdx + 1)
    text = text.slice(0, lastIdx + 1)
  } else {
    connectionPendingTexts[connectionId] = ''
  }

  if (!text) return

  const lines = text.split('\n')
  const time = new Date().toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    if (line.length === 0 && i === lines.length - 1 && !force) continue

    let content = line
    if (content.endsWith('\r')) {
      content = content.slice(0, -1)
    }

    addConnectionLog(connectionId, { type: 'rx', content, time })
    addReceivedData(connectionId, content)

    // 触发脚本行回调
    scriptLineCallbacks[connectionId]?.forEach(cb => cb(content))
    anyScriptLineCallbacks.forEach(cb => cb(connectionId, content))
  }
}

function scheduleConnectionFlush(connectionId: string) {
  if (connectionFlushTimers[connectionId]) {
    clearTimeout(connectionFlushTimers[connectionId]!)
  }
  connectionFlushTimers[connectionId] = window.setTimeout(() => {
    flushConnectionLines(connectionId, true)
    connectionFlushTimers[connectionId] = null
  }, 50)
}

/** 添加接收数据到缓存 */
export function addReceivedData(connectionId: string, content: string) {
  receivedBuffer.value.push({ connection_id: connectionId, content, time: Date.now() })
  // 限制缓存大小
  const over = receivedBuffer.value.length - maxBufferSize.value
  if (over > 0) {
    receivedBuffer.value.splice(0, over)
  }
}

/** 清空接收缓存 */
export function clearReceivedBuffer() {
  receivedBuffer.value = []
}

// ========== 标签页状态 ==========

export const serialTabs = ref<SerialTab[]>([])
export const activeTabId = ref<string | null>(null)

export const activeTab = computed(() =>
  serialTabs.value.find(t => t.id === activeTabId.value) || null
)

/** 生成默认配置（从 AppConfig 或硬编码默认值） */
export function createDefaultConfig(): SerialPortConfig {
  if (appConfig.value) {
    return {
      port_name: appConfig.value.serial.port,
      baud_rate: appConfig.value.serial.baud_rate,
      data_bits: appConfig.value.serial.data_bits,
      stop_bits: appConfig.value.serial.stop_bits,
      parity: appConfig.value.serial.parity,
      flow_control: appConfig.value.serial.flow_control,
      timeout_ms: appConfig.value.serial.timeout_ms || 100,
    }
  }
  return {
    port_name: '',
    baud_rate: 115200,
    data_bits: 8,
    stop_bits: 1,
    parity: 'None',
    flow_control: 'None',
    timeout_ms: 100,
  }
}

let tabCounter = 1

/** 添加新标签页 */
export function addSerialTab(): string {
  const lastTab = serialTabs.value[serialTabs.value.length - 1]
  const config = lastTab
    ? { ...lastTab.config }
    : createDefaultConfig()
  const id = `tab_${Date.now()}_${tabCounter++}`
  const tab: SerialTab = {
    id,
    connectionId: null,
    name: config.port_name || `Tab ${serialTabs.value.length + 1}`,
    config,
  }
  serialTabs.value.push(tab)
  activeTabId.value = id
  return id
}

/** 移除标签页（如有连接则先关闭） */
export async function removeSerialTab(tabId: string): Promise<void> {
  const idx = serialTabs.value.findIndex(t => t.id === tabId)
  if (idx < 0) return
  const tab = serialTabs.value[idx]

  if (tab.connectionId) {
    try {
      await closeSerialPort(tab.connectionId)
    } catch (e) {
      console.error('关闭串口失败:', e)
    }
    // 清理该连接的日志和显示设置
    delete connectionLogs.value[tab.connectionId]
    delete connectionDisplay.value[tab.connectionId]
  }

  serialTabs.value.splice(idx, 1)
  if (activeTabId.value === tabId) {
    const next = serialTabs.value[Math.min(idx, serialTabs.value.length - 1)]
    activeTabId.value = next?.id || null
    currentConnectionId.value = next?.connectionId || null
  }
}

/** 将标签页与连接关联 */
export function linkTabToConnection(tabId: string, connectionId: string) {
  const tab = serialTabs.value.find(t => t.id === tabId)
  if (!tab) return
  tab.connectionId = connectionId
  if (activeTabId.value === tabId) {
    currentConnectionId.value = connectionId
  }
}

/** 更新标签页名称 */
export function updateTabName(tabId: string, name: string) {
  const tab = serialTabs.value.find(t => t.id === tabId)
  if (tab) tab.name = name
}

/** 根据连接 ID 查找标签页 */
export function findTabByConnectionId(connectionId: string): SerialTab | undefined {
  return serialTabs.value.find(t => t.connectionId === connectionId)
}

// ========== 终端日志状态（按 connectionId 隔离） ==========

export const connectionLogs = ref<Record<string, LogEntry[]>>({})
export const connectionDisplay = ref<Record<string, DisplaySettings>>({})

export function getConnectionLog(connectionId: string): LogEntry[] {
  return connectionLogs.value[connectionId] || []
}

export function addConnectionLog(connectionId: string, entry: LogEntry) {
  if (!connectionLogs.value[connectionId]) {
    connectionLogs.value[connectionId] = []
  }
  connectionLogs.value[connectionId].push(entry)
  const max = maxBufferSize.value
  if (connectionLogs.value[connectionId].length > max) {
    connectionLogs.value[connectionId] = connectionLogs.value[connectionId].slice(-max)
  }
}

export function clearConnectionLog(connectionId: string) {
  connectionLogs.value[connectionId] = []
}

export function getConnectionDisplay(connectionId: string | null): DisplaySettings {
  if (!connectionId) {
    return { hexDisplay: false, encoding: 'utf-8', autoScroll: true }
  }
  if (!connectionDisplay.value[connectionId]) {
    connectionDisplay.value[connectionId] = { hexDisplay: false, encoding: 'utf-8', autoScroll: true }
  }
  return connectionDisplay.value[connectionId]
}

export function setConnectionDisplay(connectionId: string, settings: Partial<DisplaySettings>) {
  const current = getConnectionDisplay(connectionId)
  connectionDisplay.value[connectionId] = { ...current, ...settings }
}

// ========== 工具函数 ==========

/** 生成唯一的连接 ID */
export function generateConnectionId(): string {
  return `conn_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`
}

/** 从 AppConfig 创建 SerialPortConfig */
export function createConfigFromApp(portName?: string): SerialPortConfig {
  if (!appConfig.value) {
    throw new Error('配置未加载')
  }

  return {
    port_name: portName || appConfig.value.serial.port,
    baud_rate: appConfig.value.serial.baud_rate,
    data_bits: appConfig.value.serial.data_bits,
    stop_bits: appConfig.value.serial.stop_bits,
    parity: appConfig.value.serial.parity,
    flow_control: appConfig.value.serial.flow_control,
    timeout_ms: appConfig.value.serial.timeout_ms || 100,
  }
}

// ========== 串口操作 ==========

/** 刷新可用串口列表 */
export async function refreshPorts() {
  try {
    const ports = await invoke<PortInfo[]>('refresh_serial_ports')
    console.log('串口列表已刷新:', ports)
    await updateGlobalInfo()
  } catch (error) {
    console.error('刷新串口列表失败:', error)
    throw error
  }
}

/** 打开新的串口连接 */
export async function openSerialPort(
  connectionId: string,
  config: SerialPortConfig
): Promise<void> {
  try {
    await invoke('open_serial_port', {
      connectionId,
      config,
    })

    console.log(`串口连接 [${connectionId}] 已打开:`, config.port_name)

    // 更新全局信息
    await updateGlobalInfo()

    // 设置为当前连接
    currentConnectionId.value = connectionId
  } catch (error) {
    console.error('打开串口失败:', error)
    throw error
  }
}

/** 使用默认配置打开串口（从 AppConfig） */
export async function openDefaultSerialPort(): Promise<string> {
  const connectionId = generateConnectionId()
  const config = createConfigFromApp()

  await openSerialPort(connectionId, config)
  return connectionId
}

/** 关闭指定串口连接 */
export async function closeSerialPort(connectionId: string): Promise<void> {
  try {
    await invoke('close_serial_port', { connectionId })
    console.log(`串口连接 [${connectionId}] 已关闭`)

    // 清理行缓冲状态
    if (connectionFlushTimers[connectionId]) {
      clearTimeout(connectionFlushTimers[connectionId]!)
      connectionFlushTimers[connectionId] = null
    }
    delete connectionPendingTexts[connectionId]
    delete connectionByteDecoders[connectionId]
    scriptLineCallbacks[connectionId] = []

    // 如果是当前连接，切换到其他连接
    if (currentConnectionId.value === connectionId) {
      const remaining = activeConnections.value.filter(c => c.connection_id !== connectionId)
      currentConnectionId.value = remaining.length > 0 ? remaining[0].connection_id : null
    }

    await updateGlobalInfo()
  } catch (error) {
    console.error('关闭串口失败:', error)
    throw error
  }
}

/** 关闭所有串口连接 */
export async function closeAllSerialPorts(): Promise<void> {
  try {
    await invoke('close_all_serial_ports')
    console.log('所有串口已关闭')

    currentConnectionId.value = null
    await updateGlobalInfo()
  } catch (error) {
    console.error('关闭所有串口失败:', error)
    throw error
  }
}

/** 获取指定连接的信息 */
export async function getConnectionInfo(connectionId: string): Promise<ConnectionInfo> {
  try {
    return await invoke<ConnectionInfo>('get_connection_info', { connectionId })
  } catch (error) {
    console.error('获取连接信息失败:', error)
    throw error
  }
}

/** 更新全局运行时信息 */
export async function updateGlobalInfo(): Promise<void> {
  try {
    globalInfo.value = await invoke<GlobalRuntimeInfo>('get_global_runtime_info')
  } catch (error) {
    console.error('获取全局信息失败:', error)
  }
}

/** 发送数据到指定连接 */
export async function sendData(
  connectionId: string,
  data: string,
  isHex: boolean = false
): Promise<number> {
  try {
    let bytes: number[]

    if (isHex) {
      // 十六进制模式：移除所有空白字符后按每2字符解析
      const cleaned = data.replace(/\s/g, '')
      if (cleaned.length % 2 !== 0) {
        throw new Error('Hex 数据长度必须为偶数')
      }
      bytes = cleaned.match(/.{1,2}/g)?.map(byte => parseInt(byte, 16)) || []
    } else {
      // 文本模式
      bytes = Array.from(new TextEncoder().encode(data))
    }

    const sentBytes = await invoke<number>('send_serial_data', {
      connectionId,
      data: bytes,
    })

    // 自动记录终端 TX 日志（脚本发送也能看到）
    const time = new Date().toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })
    addConnectionLog(connectionId, {
      type: 'tx',
      content: data,
      time,
    })

    console.log(`[${connectionId}] 已发送 ${sentBytes} 字节`)

    // 更新状态
    await updateGlobalInfo()

    return sentBytes
  } catch (error) {
    console.error('发送数据失败:', error)
    throw error
  }
}

/** 发送数据到当前连接（兼容旧代码） */
export async function sendDataToCurrent(
  data: string,
  isHex: boolean = false
): Promise<number> {
  if (!currentConnectionId.value) {
    throw new Error('没有活跃的串口连接')
  }
  return sendData(currentConnectionId.value, data, isHex)
}

/** 发送数据到指定连接（带 CRC/校验和） */
export async function sendDataWithCrc(
  connectionId: string,
  data: string,
  isHex: boolean = false,
  crcAlgorithm: string = 'modbus'
): Promise<number> {
  try {
    let bytes: number[]
    if (isHex) {
      const cleaned = data.replace(/\s/g, '')
      if (cleaned.length % 2 !== 0) {
        throw new Error('Hex 数据长度必须为偶数')
      }
      bytes = cleaned.match(/.{1,2}/g)?.map(byte => parseInt(byte, 16)) || []
    } else {
      bytes = Array.from(new TextEncoder().encode(data))
    }

    const sentBytes = await invoke<number>('send_serial_data_with_crc', {
      connectionId,
      data: bytes,
      crcAlgorithm,
    })

    const time = new Date().toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })
    addConnectionLog(connectionId, {
      type: 'tx',
      content: `${data} [CRC:${crcAlgorithm.toUpperCase()}]`,
      time,
    })

    await updateGlobalInfo()
    return sentBytes
  } catch (error) {
    console.error('发送数据（带CRC）失败:', error)
    throw error
  }
}

/** 发送文件到指定连接 */
export async function sendFile(
  connectionId: string,
  data: Uint8Array,
  chunkSize: number = 256,
  delayMs: number = 5,
): Promise<number> {
  try {
    const sentBytes = await invoke<number>('send_serial_file', {
      connectionId,
      data: Array.from(data),
      chunkSize,
      delayMs,
    })

    const time = new Date().toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })
    addConnectionLog(connectionId, {
      type: 'tx',
      content: `[FILE] ${sentBytes} bytes`,
      time,
    })

    console.log(`[${connectionId}] 已发送文件 ${sentBytes} 字节`)

    await updateGlobalInfo()
    return sentBytes
  } catch (error) {
    console.error('发送文件失败:', error)
    throw error
  }
}

/** 检查指定连接是否已连接 */
export async function isConnected(connectionId: string): Promise<boolean> {
  try {
    return await invoke<boolean>('is_serial_connected', { connectionId })
  } catch (error) {
    console.error('检查连接状态失败:', error)
    return false
  }
}

// ========== 事件监听（接收后端推送的串口数据） ==========

/** 串口数据事件载荷 */
interface SerialDataPayload {
  connection_id: string
  data: number[]
}

/** 串口数据回调类型（传递原始字节，由组件负责解码） */
type SerialDataCallback = (connectionId: string, rawData: Uint8Array) => void
const dataCallbacks: SerialDataCallback[] = []

let unlistenSerialData: UnlistenFn | null = null

/** 启动串口数据事件监听 */
export async function startSerialDataListener() {
  if (unlistenSerialData) return

  unlistenSerialData = await listen<SerialDataPayload>('serial-data', (event) => {
    const { connection_id, data } = event.payload
    const rawData = new Uint8Array(data)

    // 使用流式解码器按行缓冲，避免串口流式数据被拆成多行
    const encoding = getConnectionDisplay(connection_id).encoding
    if (!connectionByteDecoders[connection_id] || connectionByteDecoders[connection_id].encoding !== encoding) {
      connectionByteDecoders[connection_id] = new TextDecoder(encoding, { fatal: false })
    }
    const decoder = connectionByteDecoders[connection_id]
    const chunkText = decoder.decode(rawData, { stream: true })

    connectionPendingTexts[connection_id] = (connectionPendingTexts[connection_id] || '') + chunkText
    flushConnectionLines(connection_id)
    scheduleConnectionFlush(connection_id)

    // 传递原始字节给回调（供 ChartView / 其他组件使用）
    dataCallbacks.forEach(cb => cb(connection_id, rawData))
  })
  console.log('串口数据监听已启动')
}

/** 停止串口数据事件监听 */
export function stopSerialDataListener() {
  if (unlistenSerialData) {
    unlistenSerialData()
    unlistenSerialData = null
    console.log('串口数据监听已停止')
  }
}

/** 注册串口数据回调（返回取消函数） */
export function onSerialData(callback: SerialDataCallback): () => void {
  dataCallbacks.push(callback)
  return () => {
    const idx = dataCallbacks.indexOf(callback)
    if (idx >= 0) dataCallbacks.splice(idx, 1)
  }
}

// ========== 自动更新 ==========

let pollingInterval: number | null = null

/** 开始定时更新状态 */
export function startStatusPolling(interval: number = 1000) {
  if (pollingInterval !== null) return

  pollingInterval = window.setInterval(updateGlobalInfo, interval)
  console.log('状态轮询已启动')
}

/** 停止定时更新 */
export function stopStatusPolling() {
  if (pollingInterval !== null) {
    clearInterval(pollingInterval)
    pollingInterval = null
    console.log('状态轮询已停止')
  }
}

/** 辅助：根据标签页 ID 获取连接状态 */
export function getTabConnectionStatus(tab: SerialTab): PortStatus | null {
  if (!tab.connectionId) return 'Disconnected'
  const conn = activeConnections.value.find(c => c.connection_id === tab.connectionId)
  return conn?.status || 'Disconnected'
}
