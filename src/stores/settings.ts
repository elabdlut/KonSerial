// 全局设置状态 — 从 appConfig 派生响应式设置，修改即时生效
import { computed, watch, ref } from 'vue'
import { zhCN, dateZhCN, enUS, dateEnUS } from 'naive-ui'
import type { GlobalThemeOverrides } from 'naive-ui'
import { appConfig, saveConfig } from './config'

// ========== 主题 ==========

/** 系统是否偏好暗色 */
const systemPrefersDark = ref(
  window.matchMedia('(prefers-color-scheme: dark)').matches
)

// 监听系统主题变化
window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
  systemPrefersDark.value = e.matches
})

/** 当前主题设置值 */
export const themeSetting = computed({
  get: () => appConfig.value?.ui.theme ?? 'light',
  set: (v: string) => {
    if (appConfig.value) appConfig.value.ui.theme = v
  },
})

/** 当前是否实际使用暗色主题 */
export const isDark = computed(() => {
  if (themeSetting.value === 'dark') return true
  if (themeSetting.value === 'auto') return systemPrefersDark.value
  return false
})

// ========== 字体大小 ==========

export const fontSize = computed({
  get: () => appConfig.value?.ui.font_size ?? 14,
  set: (v: number) => {
    if (appConfig.value) appConfig.value.ui.font_size = v
  },
})

/** Naive UI 全局字体大小覆盖 — 所有组件文字大小随设置变化 */
export const naiveThemeOverrides = computed<GlobalThemeOverrides>(() => {
  const base = fontSize.value
  return {
    common: {
      fontSizeMini: `${base - 2}px`,
      fontSizeTiny: `${base - 2}px`,
      fontSizeSmall: `${base}px`,
      fontSizeMedium: `${base}px`,
      fontSizeLarge: `${base + 1}px`,
      fontSizeHuge: `${base + 2}px`,
    },
  }
})

// ========== 数据设置 ==========

export const maxBufferSize = computed({
  get: () => appConfig.value?.data.max_buffer_size ?? 10000,
  set: (v: number) => {
    if (appConfig.value) appConfig.value.data.max_buffer_size = v
  },
})

export const autoSave = computed({
  get: () => appConfig.value?.data.auto_save ?? true,
  set: (v: boolean) => {
    if (appConfig.value) appConfig.value.data.auto_save = v
  },
})

export const saveInterval = computed({
  get: () => appConfig.value?.data.save_interval ?? 60,
  set: (v: number) => {
    if (appConfig.value) appConfig.value.data.save_interval = v
  },
})

// ========== 语言 ==========

export const language = computed({
  get: () => appConfig.value?.ui.language ?? 'zh-CN',
  set: (v: string) => {
    if (appConfig.value) appConfig.value.ui.language = v
  },
})

/** Naive UI locale — 响应式切换中英文 */
export const naiveLocale = computed(() =>
  language.value === 'en-US' ? enUS : zhCN
)

export const naiveDateLocale = computed(() =>
  language.value === 'en-US' ? dateEnUS : dateZhCN
)

// ========== 副作用：主题应用到 DOM ==========

/** 将主题类名应用到 document.documentElement */
export function applyThemeToDOM() {
  watch(isDark, (dark) => {
    if (dark) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }, { immediate: true })
}

/** 将字体大小应用到 CSS 变量 */
export function applyFontSizeToDOM() {
  watch(fontSize, (size) => {
    document.documentElement.style.setProperty('--app-font-size', `${size}px`)
  }, { immediate: true })
}

// ========== 网络设置 ==========

export const networkProtocol = computed({
  get: () => appConfig.value?.network.protocol ?? 'tcp',
  set: (v: string) => {
    if (appConfig.value) appConfig.value.network.protocol = v
  },
})

export const networkHost = computed({
  get: () => appConfig.value?.network.host ?? '127.0.0.1',
  set: (v: string) => {
    if (appConfig.value) appConfig.value.network.host = v
  },
})

export const networkPort = computed({
  get: () => appConfig.value?.network.port ?? 8080,
  set: (v: number) => {
    if (appConfig.value) appConfig.value.network.port = v
  },
})

export const networkAutoReconnect = computed({
  get: () => appConfig.value?.network.auto_reconnect ?? false,
  set: (v: boolean) => {
    if (appConfig.value) appConfig.value.network.auto_reconnect = v
  },
})

export const networkReconnectInterval = computed({
  get: () => appConfig.value?.network.reconnect_interval_ms ?? 1000,
  set: (v: number) => {
    if (appConfig.value) appConfig.value.network.reconnect_interval_ms = v
  },
})

export const networkMaxReconnectAttempts = computed({
  get: () => appConfig.value?.network.max_reconnect_attempts ?? 3,
  set: (v: number) => {
    if (appConfig.value) appConfig.value.network.max_reconnect_attempts = v
  },
})

// ========== 保存 ==========

/** 持久化当前设置到磁盘 */
export async function persistSettings() {
  await saveConfig()
}
