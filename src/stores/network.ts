// 网络调试状态管理
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { addReceivedData } from './serial'
import { appConfig } from './config'

// ========== 类型定义 ==========

export type NetProtocol = 'tcp' | 'udp' | 'ws' | 'mqtt' | 'tcp_server' | 'udp_server'

export interface NetConnectionConfig {
  protocol: NetProtocol
  host: string
  port: number
  path?: string
  topic?: string
  auto_reconnect?: boolean
  reconnect_interval_ms?: number
  max_reconnect_attempts?: number
}

export type NetStatus = 'Disconnected' | 'Connecting' | 'Connected' | 'Listening' | { Error: string }

export interface NetConnectionInfo {
  connection_id: string
  status: NetStatus
  config: NetConnectionConfig
  bytes_received: number
  bytes_sent: number
  rx_rate: number
  tx_rate: number
  connected_at: string | null
  last_error: string | null
  created_at: string
}

export interface NetworkGlobalInfo {
  active_connections: NetConnectionInfo[]
  total_connections: number
}

export interface LogEntry {
  type: string
  content: string
  time: string
}

export interface NetworkTab {
  id: string
  connectionId: string | null
  name: string
  config: NetConnectionConfig
}

export interface NetworkPeerEvent {
  connection_id: string
  peer_id: string
  event: 'connected' | 'disconnected'
}

// ========== 响应式状态 ==========

export const networkTabs = ref<NetworkTab[]>([])
export const activeTabId = ref<string | null>(null)

export const activeTab = computed(() =>
  networkTabs.value.find(t => t.id === activeTabId.value) || null
)

export const globalInfo = ref<NetworkGlobalInfo | null>(null)

export const activeConnections = computed(() => globalInfo.value?.active_connections || [])

// Peer 管理
export const connectionPeers = ref<Record<string, string[]>>({})
export const selectedPeer = ref<Record<string, string | null>>({})

export function addPeer(connectionId: string, peerId: string) {
  const list = connectionPeers.value[connectionId] || []
  if (!list.includes(peerId)) {
    connectionPeers.value[connectionId] = [...list, peerId]
  }
  if (!selectedPeer.value[connectionId]) {
    selectedPeer.value[connectionId] = peerId
  }
}

export function removePeer(connectionId: string, peerId: string) {
  const list = connectionPeers.value[connectionId] || []
  connectionPeers.value[connectionId] = list.filter(id => id !== peerId)
  if (selectedPeer.value[connectionId] === peerId) {
    const remaining = connectionPeers.value[connectionId]
    selectedPeer.value[connectionId] = remaining.length > 0 ? remaining[0] : null
  }
}

export function clearPeers(connectionId: string) {
  delete connectionPeers.value[connectionId]
  delete selectedPeer.value[connectionId]
}

// 自动重连状态跟踪
const reconnectingIds = new Set<string>()
const reconnectAttemptCounts: Record<string, number> = {}

watch(
  activeConnections,
  (conns) => {
    for (const conn of conns) {
      // Server 模式不参与自动重连
      if (conn.config.protocol.endsWith('_server')) continue

      if (typeof conn.status !== 'string') {
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
              openNetworkConnection(conn.connection_id, latest.config).catch((e) => {
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
        delete reconnectAttemptCounts[conn.connection_id]
      }
    }
  }
)

// ========== 工具函数 ==========

let tabCounter = 1

export function createDefaultNetConfig(): NetConnectionConfig {
  const netCfg = appConfig.value?.network
  if (netCfg) {
    return {
      protocol: netCfg.protocol as NetProtocol,
      host: netCfg.host,
      port: netCfg.port,
      auto_reconnect: netCfg.auto_reconnect,
      reconnect_interval_ms: netCfg.reconnect_interval_ms,
      max_reconnect_attempts: netCfg.max_reconnect_attempts,
    }
  }
  return {
    protocol: 'tcp',
    host: '127.0.0.1',
    port: 8080,
  }
}

export function addNetworkTab(): string {
  const lastTab = networkTabs.value[networkTabs.value.length - 1]
  const config = lastTab
    ? { ...lastTab.config }
    : createDefaultNetConfig()
  const id = `net_tab_${Date.now()}_${tabCounter++}`
  const tab: NetworkTab = {
    id,
    connectionId: null,
    name: `${config.protocol.toUpperCase()} ${networkTabs.value.length + 1}`,
    config,
  }
  networkTabs.value.push(tab)
  activeTabId.value = id
  return id
}

export async function removeNetworkTab(tabId: string): Promise<void> {
  const idx = networkTabs.value.findIndex(t => t.id === tabId)
  if (idx < 0) return
  const tab = networkTabs.value[idx]

  if (tab.connectionId) {
    try {
      await closeNetworkConnection(tab.connectionId)
    } catch (e) {
      console.error('关闭网络连接失败:', e)
    }
    delete connectionLogs.value[tab.connectionId]
    delete connectionDisplay.value[tab.connectionId]
    clearPeers(tab.connectionId)
  }

  networkTabs.value.splice(idx, 1)
  if (activeTabId.value === tabId) {
    const next = networkTabs.value[Math.min(idx, networkTabs.value.length - 1)]
    activeTabId.value = next?.id || null
  }
}

export function linkTabToConnection(tabId: string, connectionId: string) {
  const tab = networkTabs.value.find(t => t.id === tabId)
  if (!tab) return
  tab.connectionId = connectionId
}

export function updateTabName(tabId: string, name: string) {
  const tab = networkTabs.value.find(t => t.id === tabId)
  if (tab) tab.name = name
}

export function generateConnectionId(): string {
  return `net_conn_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`
}

// ========== 终端日志状态 ==========

export const connectionLogs = ref<Record<string, LogEntry[]>>({})
export const connectionDisplay = ref<Record<string, { hexDisplay: boolean; autoScroll: boolean; encoding: string }>>({})

export function getConnectionLog(connectionId: string): LogEntry[] {
  return connectionLogs.value[connectionId] || []
}

export function addConnectionLog(connectionId: string, entry: LogEntry) {
  if (!connectionLogs.value[connectionId]) {
    connectionLogs.value[connectionId] = []
  }
  connectionLogs.value[connectionId].push(entry)
  const max = 10000
  if (connectionLogs.value[connectionId].length > max) {
    connectionLogs.value[connectionId] = connectionLogs.value[connectionId].slice(-max)
  }
}

export function clearConnectionLog(connectionId: string) {
  connectionLogs.value[connectionId] = []
}

export function getConnectionDisplay(connectionId: string | null) {
  if (!connectionId) {
    return { hexDisplay: false, autoScroll: true, encoding: 'utf-8' }
  }
  if (!connectionDisplay.value[connectionId]) {
    connectionDisplay.value[connectionId] = { hexDisplay: false, autoScroll: true, encoding: 'utf-8' }
  }
  return connectionDisplay.value[connectionId]
}

export function setConnectionDisplay(connectionId: string, settings: Partial<{ hexDisplay: boolean; autoScroll: boolean; encoding: string }>) {
  const current = getConnectionDisplay(connectionId)
  connectionDisplay.value[connectionId] = { ...current, ...settings }
}

// ========== 行缓冲状态（支持脚本按行回调） ==========

const connectionPendingTexts: Record<string, string> = {}
const connectionFlushTimers: Record<string, number | null> = {}

type ScriptLineCallback = (line: string) => void
type AnyScriptLineCallback = (connectionId: string, line: string) => void
const scriptLineCallbacks: Record<string, ScriptLineCallback[]> = {}
const anyScriptLineCallbacks: AnyScriptLineCallback[] = []

export function onNetworkScriptDataLine(connectionId: string, callback: ScriptLineCallback): () => void {
  if (!scriptLineCallbacks[connectionId]) scriptLineCallbacks[connectionId] = []
  scriptLineCallbacks[connectionId].push(callback)
  return () => {
    const arr = scriptLineCallbacks[connectionId]
    if (!arr) return
    const idx = arr.indexOf(callback)
    if (idx >= 0) arr.splice(idx, 1)
  }
}

export function onAnyNetworkScriptDataLine(callback: AnyScriptLineCallback): () => void {
  anyScriptLineCallbacks.push(callback)
  return () => {
    const idx = anyScriptLineCallbacks.indexOf(callback)
    if (idx >= 0) anyScriptLineCallbacks.splice(idx, 1)
  }
}

function getBufferKey(connectionId: string, peerId?: string): string {
  return `${connectionId}#${peerId || '_'}`
}

function flushNetworkLines(connectionId: string, peerId?: string, force = false) {
  const key = getBufferKey(connectionId, peerId)
  let text = connectionPendingTexts[key] || ''
  if (!text) return

  if (!force) {
    const lastIdx = text.lastIndexOf('\n')
    if (lastIdx === -1) return
    connectionPendingTexts[key] = text.slice(lastIdx + 1)
    text = text.slice(0, lastIdx + 1)
  } else {
    connectionPendingTexts[key] = ''
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

    const displayContent = peerId ? `[${peerId}] ${content}` : content
    addConnectionLog(connectionId, { type: 'rx', content: displayContent, time })
    addReceivedData(connectionId, displayContent)

    scriptLineCallbacks[connectionId]?.forEach(cb => cb(displayContent))
    anyScriptLineCallbacks.forEach(cb => cb(connectionId, displayContent))
  }
}

function scheduleNetworkFlush(connectionId: string, peerId?: string) {
  const key = getBufferKey(connectionId, peerId)
  if (connectionFlushTimers[key]) {
    clearTimeout(connectionFlushTimers[key]!)
  }
  connectionFlushTimers[key] = window.setTimeout(() => {
    flushNetworkLines(connectionId, peerId, true)
    connectionFlushTimers[key] = null
  }, 50)
}

// ========== 网络操作 ==========

export async function openNetworkConnection(
  connectionId: string,
  config: NetConnectionConfig
): Promise<void> {
  try {
    await invoke('open_network_connection', { connectionId, config })
    console.log(`网络连接 [${connectionId}] 已打开:`, config)
    await updateGlobalInfo()
  } catch (error) {
    console.error('打开网络连接失败:', error)
    throw error
  }
}

export async function closeNetworkConnection(connectionId: string): Promise<void> {
  try {
    await invoke('close_network_connection', { connectionId })
    console.log(`网络连接 [${connectionId}] 已关闭`)
    clearPeers(connectionId)
    await updateGlobalInfo()
  } catch (error) {
    console.error('关闭网络连接失败:', error)
    throw error
  }
}

export async function sendNetworkData(
  connectionId: string,
  data: string,
  isHex: boolean = false,
  peerId?: string
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

    const sentBytes = await invoke<number>('send_network_data', {
      connectionId,
      data: bytes,
      peerId: peerId || null,
    })

    const time = new Date().toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })
    const displayContent = peerId ? `[${peerId}] ${data}` : data
    addConnectionLog(connectionId, {
      type: 'tx',
      content: displayContent,
      time,
    })

    await updateGlobalInfo()
    return sentBytes
  } catch (error) {
    console.error('发送网络数据失败:', error)
    throw error
  }
}

export async function sendNetworkFile(
  connectionId: string,
  bytes: Uint8Array,
  peerId?: string
): Promise<number> {
  try {
    const data = Array.from(bytes)
    const sentBytes = await invoke<number>('send_network_data', {
      connectionId,
      data,
      peerId: peerId || null,
    })
    await updateGlobalInfo()
    return sentBytes
  } catch (error) {
    console.error('发送网络文件失败:', error)
    throw error
  }
}

export async function updateGlobalInfo(): Promise<void> {
  try {
    globalInfo.value = await invoke<NetworkGlobalInfo>('get_network_global_info')
  } catch (error) {
    console.error('获取网络全局信息失败:', error)
  }
}

// ========== 事件监听 ==========

interface NetworkDataPayload {
  connection_id: string
  data: number[]
  peer_id?: string
}

let unlistenNetworkData: UnlistenFn | null = null
let unlistenPeerUpdate: UnlistenFn | null = null

export async function startNetworkDataListener() {
  if (unlistenNetworkData) return

  unlistenNetworkData = await listen<NetworkDataPayload>('network-data', (event) => {
    const { connection_id, data, peer_id } = event.payload
    const rawData = new Uint8Array(data)
    const encoding = getConnectionDisplay(connection_id).encoding
    const decoder = new TextDecoder(encoding, { fatal: false })
    const text = decoder.decode(rawData)

    const key = getBufferKey(connection_id, peer_id)
    connectionPendingTexts[key] = (connectionPendingTexts[key] || '') + text
    flushNetworkLines(connection_id, peer_id)
    scheduleNetworkFlush(connection_id, peer_id)
  })

  unlistenPeerUpdate = await listen<NetworkPeerEvent>('network-peer-update', (event) => {
    const { connection_id, peer_id, event: evt } = event.payload
    if (evt === 'connected') {
      addPeer(connection_id, peer_id)
      const time = new Date().toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })
      addConnectionLog(connection_id, { type: 'system', content: `客户端连接: ${peer_id}`, time })
    } else {
      removePeer(connection_id, peer_id)
      const time = new Date().toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })
      addConnectionLog(connection_id, { type: 'system', content: `客户端断开: ${peer_id}`, time })
    }
  })

  console.log('网络数据监听已启动')
}

export function stopNetworkDataListener() {
  if (unlistenNetworkData) {
    unlistenNetworkData()
    unlistenNetworkData = null
  }
  if (unlistenPeerUpdate) {
    unlistenPeerUpdate()
    unlistenPeerUpdate = null
  }
  console.log('网络数据监听已停止')
}

// ========== 辅助函数 ==========

export function getTabConnectionStatus(tab: NetworkTab): NetStatus | null {
  if (!tab.connectionId) return 'Disconnected'
  const conn = activeConnections.value.find(c => c.connection_id === tab.connectionId)
  return conn?.status || 'Disconnected'
}

let pollingInterval: number | null = null

export function startStatusPolling(interval: number = 1000) {
  if (pollingInterval !== null) return
  pollingInterval = window.setInterval(updateGlobalInfo, interval)
  console.log('网络状态轮询已启动')
}

export function stopStatusPolling() {
  if (pollingInterval !== null) {
    clearInterval(pollingInterval)
    pollingInterval = null
    console.log('网络状态轮询已停止')
  }
}
