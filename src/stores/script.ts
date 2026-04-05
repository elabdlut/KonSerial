import { ref, computed } from 'vue'
import {
  sendData, activeConnections, onScriptDataLine, onAnyScriptDataLine,
  type ConnectionInfo,
} from './serial'

export interface ScriptFile {
  id: string
  name: string
  content: string
  path: string | null
  modified: boolean
}

export interface LogEntry {
  type: 'info' | 'error' | 'success' | 'warn'
  content: string
  time: string
}

// ===== 全局持久化状态（页面切换不丢失） =====

export const scriptFiles = ref<ScriptFile[]>([
  {
    id: 'default',
    name: 'main.js',
    path: null,
    modified: false,
    content: `// 使用 TARGET 获取第一个选中的连接
// 使用 TARGETS 获取所有选中的连接数组
// serial.send(text) 默认发送到 TARGET

console.log("已选连接:", TARGETS);
`,
  },
])

export const activeScriptId = ref('default')
export const scriptLogs = ref<LogEntry[]>([])
export const scriptIsRunning = ref(false)
export const selectedConnectionIds = ref<string[]>([])

export const activeScript = computed(() =>
  scriptFiles.value.find(s => s.id === activeScriptId.value)
)

// ===== 运行时状态（内部管理） =====

let runningAbort: AbortController | null = null
let activeTimers: number[] = []
let scriptDataUnsubs: (() => void)[] = []

// ===== 日志工具 =====

export function addScriptLog(type: LogEntry['type'], content: string) {
  const now = new Date()
  const time = now.toLocaleTimeString('zh-CN', { hour12: false })
  scriptLogs.value.push({ type, content, time })
  if (scriptLogs.value.length > 500) {
    scriptLogs.value.splice(0, scriptLogs.value.length - 500)
  }
}

export function clearScriptLogs() {
  scriptLogs.value = []
}

function cleanupScript() {
  activeTimers.forEach(id => { clearInterval(id); clearTimeout(id) })
  activeTimers = []
  scriptDataUnsubs.forEach(u => u())
  scriptDataUnsubs = []
  runningAbort = null
}

export function stopScript() {
  if (!scriptIsRunning.value) return
  runningAbort?.abort()
  cleanupScript()
  scriptIsRunning.value = false
  addScriptLog('info', '脚本已停止')
}

// ===== 脚本模板 =====

export interface ScriptTemplate {
  label: string
  value: string
  content: string
}

export const scriptTemplates: ScriptTemplate[] = [
  {
    label: '空模板',
    value: 'empty',
    content: `// 使用 TARGET 获取第一个选中的连接\n// 使用 TARGETS 获取所有选中的连接数组\n// serial.send(text) 默认发送到 TARGET\n\nconsole.log("已选连接:", TARGETS);\n`,
  },
  {
    label: '定时发送数据',
    value: 'timer',
    content: `// 场景：每隔一段时间向某个串口发送数据\n\nlet count = 0;\nconst timer = setInterval(async () => {\n  count++;\n  await serial.send(\`heartbeat:\${count}\\n\`);\n  console.log(\`已发送心跳 #\${count}\`);\n}, 2000);\n\nconsole.log("定时发送已启动，点击停止以结束");\n`,
  },
  {
    label: '批量多串口发送',
    value: 'multi',
    content: `// 场景：批量给多个串口发送初始化命令\n\nconst cmds = ["CMD_INIT\\n", "CMD_START\\n"];\n\nfor (const connId of TARGETS) {\n  for (const cmd of cmds) {\n    await serial.sendTo(connId, cmd);\n    console.log(\`发送至 \${connId}: \${cmd.trim()}\`);\n    await sleep(100);\n  }\n}\n\nconsole.log("批量发送完成");\n`,
  },
  {
    label: '循环发送多种数据',
    value: 'cycle',
    content: `// 场景：循环向串口发送多种数据\n\nconst frames = ["A\\n", "B\\n", "C\\n"];\nlet idx = 0;\n\nconst timer = setInterval(async () => {\n  const data = frames[idx];\n  await serial.send(data);\n  console.log(\`发送帧 [\${idx}]: \${data.trim()}\`);\n  idx = (idx + 1) % frames.length;\n}, 1000);\n\nconsole.log("循环发送已启动");\n`,
  },
  {
    label: '串口数据转发',
    value: 'forward',
    content: `// 场景：从一个串口接收数据，处理后转发到另一个串口\n// 请确保选择了至少两个串口连接（第一个是源，第二个是目标）\n\nconst SOURCE = TARGETS[0];\nconst DEST = TARGETS[1];\n\nif (!SOURCE || !DEST) {\n  console.error("请至少选择两个连接：源和目标");\n  return;\n}\n\nserial.onData(SOURCE, async (line) => {\n  console.log(\`[RX \${SOURCE}] \${line}\`);\n  const out = \`FWD:\${line}\\n\`;\n  await serial.sendTo(DEST, out);\n});\n\nconsole.log(\`数据转发已启动: \${SOURCE} -> \${DEST}\`);\n`,
  },
  {
    label: '条件响应（收到 A 回复 B）',
    value: 'echo',
    content: `// 场景：监听串口数据，匹配到特定内容后自动回复\n\nserial.onData(TARGET, async (line) => {\n  console.log(\`收到: \${line}\`);\n\n  if (line.includes("ping")) {\n    await serial.send("pong\\n");\n    console.log("自动回复: pong");\n  } else if (line.includes("status")) {\n    await serial.send("OK\\n");\n    console.log("自动回复: OK");\n  }\n});\n\nconsole.log("条件响应已启动");\n`,
  },
]

// ===== 执行引擎 =====

export async function runScript(
  selectedConnectionIds: string[],
  scriptContent: string,
  onNotify?: (type: 'success' | 'error' | 'warn' | 'info', msg: string) => void,
) {
  if (scriptIsRunning.value) return
  if (selectedConnectionIds.length === 0) {
    addScriptLog('warn', '请先连接串口')
    onNotify?.('warn', '请先连接串口')
    return
  }

  scriptIsRunning.value = true
  addScriptLog('info', '脚本开始执行')
  onNotify?.('success', '脚本已启动')

  runningAbort = new AbortController()
  const primaryTarget = selectedConnectionIds[0]
  const allTargets = [...selectedConnectionIds]

  const serialApi = {
    send: async (data: string) => {
      if (!runningAbort || runningAbort.signal.aborted) return
      try {
        addScriptLog('info', `TX[${primaryTarget.slice(-6)}] -> ${data.replace(/\n/g, '\\n').replace(/\r/g, '\\r')}`)
        await sendData(primaryTarget, data, false)
      } catch (e) {
        addScriptLog('error', `TX fail: ${String(e)}`)
        throw e
      }
    },
    sendHex: async (hex: string) => {
      if (!runningAbort || runningAbort.signal.aborted) return
      try {
        addScriptLog('info', `TX[${primaryTarget.slice(-6)}](HEX) -> ${hex}`)
        await sendData(primaryTarget, hex, true)
      } catch (e) {
        addScriptLog('error', `TX fail: ${String(e)}`)
        throw e
      }
    },
    sendTo: async (connectionId: string, data: string) => {
      if (!runningAbort || runningAbort.signal.aborted) return
      try {
        addScriptLog('info', `TX[${connectionId.slice(-6)}] -> ${data.replace(/\n/g, '\\n').replace(/\r/g, '\\r')}`)
        await sendData(connectionId, data, false)
      } catch (e) {
        addScriptLog('error', `TX fail: ${String(e)}`)
        throw e
      }
    },
    sendHexTo: async (connectionId: string, hex: string) => {
      if (!runningAbort || runningAbort.signal.aborted) return
      try {
        addScriptLog('info', `TX[${connectionId.slice(-6)}](HEX) -> ${hex}`)
        await sendData(connectionId, hex, true)
      } catch (e) {
        addScriptLog('error', `TX fail: ${String(e)}`)
        throw e
      }
    },
    listConnections: () => {
      return activeConnections.value.map((c: ConnectionInfo) => ({
        connection_id: c.connection_id,
        port_name: c.config.port_name,
        baud_rate: c.config.baud_rate,
        status: typeof c.status === 'string' ? c.status : 'Error',
      }))
    },
    isConnected: (connectionId: string) => {
      return activeConnections.value.some(c => c.connection_id === connectionId && c.status === 'Connected')
    },
    onData: (connectionId: string, callback: (line: string) => void) => {
      const unsub = onScriptDataLine(connectionId, (line) => {
        if (runningAbort?.signal.aborted) return
        try { callback(line) } catch (e) { addScriptLog('error', String(e)) }
      })
      scriptDataUnsubs.push(unsub)
      return unsub
    },
    onAnyData: (callback: (connectionId: string, line: string) => void) => {
      const unsub = onAnyScriptDataLine((connId, line) => {
        if (runningAbort?.signal.aborted) return
        try { callback(connId, line) } catch (e) { addScriptLog('error', String(e)) }
      })
      scriptDataUnsubs.push(unsub)
      return unsub
    },
  }

  const sleepFn = (ms: number) => {
    return new Promise<void>((resolve, reject) => {
      if (runningAbort?.signal.aborted) { reject(new Error('Script stopped')); return }
      const id = window.setTimeout(() => {
        activeTimers = activeTimers.filter(t => t !== id)
        if (runningAbort?.signal.aborted) { reject(new Error('Script stopped')); return }
        resolve()
      }, ms)
      activeTimers.push(id)
      runningAbort?.signal.addEventListener('abort', () => {
        clearTimeout(id)
        reject(new Error('Script stopped'))
      })
    })
  }

  const setIntervalWrapped = (callback: () => void | Promise<void>, ms: number) => {
    if (runningAbort?.signal.aborted) return -1
    const id = window.setInterval(() => {
      if (runningAbort?.signal.aborted) { clearInterval(id); return }
      try {
        const result = callback()
        if (result && typeof (result as Promise<void>).then === 'function') {
          Promise.resolve(result).catch(e => addScriptLog('error', String(e)))
        }
      } catch (e) { addScriptLog('error', String(e)) }
    }, ms)
    activeTimers.push(id)
    return id
  }

  const setTimeoutWrapped = (callback: () => void | Promise<void>, ms: number) => {
    if (runningAbort?.signal.aborted) return -1
    const id = window.setTimeout(() => {
      activeTimers = activeTimers.filter(t => t !== id)
      if (runningAbort?.signal.aborted) return
      try {
        const result = callback()
        if (result && typeof (result as Promise<void>).then === 'function') {
          Promise.resolve(result).catch(e => addScriptLog('error', String(e)))
        }
      } catch (e) { addScriptLog('error', String(e)) }
    }, ms)
    activeTimers.push(id)
    return id
  }

  const clearIntervalWrapped = (id: number) => {
    clearInterval(id)
    clearTimeout(id)
    activeTimers = activeTimers.filter(t => t !== id)
  }

  const consoleMock = {
    log: (...args: unknown[]) => addScriptLog('info', args.map(String).join(' ')),
    warn: (...args: unknown[]) => addScriptLog('warn', args.map(String).join(' ')),
    error: (...args: unknown[]) => addScriptLog('error', args.map(String).join(' ')),
    info: (...args: unknown[]) => addScriptLog('info', args.map(String).join(' ')),
  }

  try {
    const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor
    const fn = new AsyncFunction(
      'serial', 'sleep', 'console',
      'setInterval', 'clearInterval', 'setTimeout', 'clearTimeout',
      'TARGET', 'TARGETS',
      scriptContent
    )
    await fn(
      serialApi, sleepFn, consoleMock,
      setIntervalWrapped, clearIntervalWrapped,
      setTimeoutWrapped, clearIntervalWrapped,
      primaryTarget, allTargets
    )
    if (runningAbort && !runningAbort.signal.aborted) {
      if (activeTimers.length === 0 && scriptDataUnsubs.length === 0) {
        addScriptLog('success', '脚本执行完成')
        scriptIsRunning.value = false
        runningAbort = null
      } else {
        addScriptLog('info', `脚本已注册持续任务 (timers=${activeTimers.length}, listeners=${scriptDataUnsubs.length})`)
      }
    }
  } catch (e: unknown) {
    const msg = String(e)
    if (!msg.includes('Script stopped')) {
      addScriptLog('error', `Error: ${msg}`)
      cleanupScript()
      scriptIsRunning.value = false
      runningAbort = null
    }
  }
}

// ===== 文件管理工具 =====

let nextId = 1

export function newScriptFile(): string {
  const id = `script_${Date.now()}_${nextId++}`
  scriptFiles.value.push({
    id,
    name: `untitled_${nextId}.js`,
    path: null,
    modified: false,
    content: '// New script\n\nconsole.log("Hello KonSerial!");\n',
  })
  activeScriptId.value = id
  return id
}

export function removeScriptFile(id: string) {
  if (scriptFiles.value.length <= 1) return
  const idx = scriptFiles.value.findIndex(s => s.id === id)
  if (idx < 0) return
  scriptFiles.value.splice(idx, 1)
  if (activeScriptId.value === id) {
    activeScriptId.value = scriptFiles.value[0].id
  }
}

export function setScriptModified(id: string) {
  const s = scriptFiles.value.find(s => s.id === id)
  if (s) s.modified = true
}

export function updateScriptContent(id: string, content: string) {
  const s = scriptFiles.value.find(s => s.id === id)
  if (s) {
    s.content = content
    s.modified = true
  }
}
