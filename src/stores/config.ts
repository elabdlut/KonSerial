// 配置状态管理
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// 配置接口定义
interface SerialConfig {
  port: string
  baud_rate: number
  data_bits: number
  stop_bits: number
  parity: string
  flow_control: string
}

interface UiConfig {
  theme: string
  language: string
  font_size: number
  sidebar_width: number
  window_width: number
  window_height: number
}

interface DataConfig {
  auto_save: boolean
  save_interval: number
  max_buffer_size: number
  data_format: string
}

export interface AppConfig {
  serial: SerialConfig
  ui: UiConfig
  data: DataConfig
}

// 全局配置状态（前端管理）
export const appConfig = ref<AppConfig | null>(null)

// 加载配置
export async function loadConfig() {
  try {
    appConfig.value = await invoke<AppConfig>('load_config')
    console.log('配置已加载:', appConfig.value)
  } catch (error) {
    console.error('加载配置失败:', error)
  }
}

// 保存配置
export async function saveConfig() {
  if (!appConfig.value) {
    console.error('配置未加载，无法保存')
    return
  }
  
  try {
    await invoke('save_config', { config: appConfig.value })
    console.log('配置已保存')
  } catch (error) {
    console.error('保存配置失败:', error)
  }
}

// 更新波特率
export async function updateBaudRate(rate: number) {
  if (appConfig.value) {
    appConfig.value.serial.baud_rate = rate
    await saveConfig()
  }
}

// 更新串口
export async function updatePort(port: string) {
  if (appConfig.value) {
    appConfig.value.serial.port = port
    await saveConfig()
  }
}

// 更新主题
export async function updateTheme(theme: string) {
  if (appConfig.value) {
    appConfig.value.ui.theme = theme
    await saveConfig()
  }
}
