// 串口状态管理
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { appConfig } from './config'

// 串口运行时状态接口
interface PortRuntimeInfo {
  status: 'Disconnected' | 'Connecting' | 'Connected' | { Error: string }
  connected_port: string | null
  current_baud_rate: number | null
  bytes_received: number
  bytes_sent: number
  last_error: string | null
}

interface SerialPortInfo {
  port_name: string
  port_type: string
}

// 运行时状态（从后端获取）
export const serialStatus = ref<PortRuntimeInfo | null>(null)

// 可用串口列表
export const availablePorts = ref<string[]>([])

// 刷新串口列表
export async function refreshPorts() {
  try {
    availablePorts.value = await invoke<string[]>('list_serial_ports')
    console.log('串口列表已刷新:', availablePorts.value)
  } catch (error) {
    console.error('刷新串口列表失败:', error)
  }
}

// 打开串口（使用前端配置）
export async function openSerialPort() {
  if (!appConfig.value) {
    console.error('配置未加载')
    return
  }
  
  try {
    await invoke('open_serial_port', {
      portName: appConfig.value.serial.port,
      baudRate: appConfig.value.serial.baud_rate,
    })
    
    console.log('串口已打开')
    
    // 获取最新状态
    await updateSerialStatus()
  } catch (error) {
    console.error('打开串口失败:', error)
  }
}

// 关闭串口
export async function closeSerialPort() {
  try {
    await invoke('close_serial_port')
    console.log('串口已关闭')
    
    // 更新状态
    await updateSerialStatus()
  } catch (error) {
    console.error('关闭串口失败:', error)
  }
}

// 更新串口状态
export async function updateSerialStatus() {
  try {
    serialStatus.value = await invoke<PortRuntimeInfo>('get_serial_status')
  } catch (error) {
    console.error('获取串口状态失败:', error)
  }
}

// 发送数据
export async function sendData(data: string, isHex: boolean = false) {
  try {
    let bytes: number[]
    
    if (isHex) {
      // 十六进制模式
      bytes = data.match(/.{1,2}/g)?.map(byte => parseInt(byte, 16)) || []
    } else {
      // 文本模式
      bytes = Array.from(new TextEncoder().encode(data))
    }
    
    const sentBytes = await invoke<number>('send_serial_data', { data: bytes })
    console.log(`已发送 ${sentBytes} 字节`)
    
    // 更新状态
    await updateSerialStatus()
  } catch (error) {
    console.error('发送数据失败:', error)
  }
}

// 定时更新状态（可选）
let statusInterval: number | null = null

export function startStatusPolling(interval: number = 1000) {
  if (statusInterval !== null) return
  
  statusInterval = window.setInterval(updateSerialStatus, interval)
}

export function stopStatusPolling() {
  if (statusInterval !== null) {
    clearInterval(statusInterval)
    statusInterval = null
  }
}
