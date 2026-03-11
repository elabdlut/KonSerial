// 串口状态管理（多连接架构）
import { ref, computed } from 'vue'
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

/** 当前选中的连接 ID（用于标签页切换） */
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
export const maxBufferSize = ref(10000)

/** 添加接收数据到缓存 */
export function addReceivedData(connectionId: string, content: string) {
  receivedBuffer.value.push({ connection_id: connectionId, content, time: Date.now() })
  // 限制缓存大小
  while (receivedBuffer.value.length > maxBufferSize.value) {
    receivedBuffer.value.shift()
  }
}

/** 清空接收缓存 */
export function clearReceivedBuffer() {
  receivedBuffer.value = []
}

// ========== 工具函数 ==========

/** 生成唯一的连接 ID */
export function generateConnectionId(): string {
  return `conn_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
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
      // 十六进制模式
      bytes = data.match(/.{1,2}/g)?.map(byte => parseInt(byte, 16)) || []
    } else {
      // 文本模式
      bytes = Array.from(new TextEncoder().encode(data))
    }
    
    const sentBytes = await invoke<number>('send_serial_data', {
      connectionId,
      data: bytes,
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

/** 发送数据到当前连接 */
export async function sendDataToCurrent(
  data: string,
  isHex: boolean = false
): Promise<number> {
  if (!currentConnectionId.value) {
    throw new Error('没有活跃的串口连接')
  }
  return sendData(currentConnectionId.value, data, isHex)
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

    // 传递原始字节给回调，由各组件根据选择的编码解码显示
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
