import { onMounted, onUnmounted } from 'vue'

export interface ShortcutConfig {
  key: string
  ctrl?: boolean
  shift?: boolean
  alt?: boolean
  handler: (e: KeyboardEvent) => void
  /** 仅在指定元素聚焦时触发，不传则全局触发 */
  target?: HTMLElement | null
}

/**
 * 注册键盘快捷键
 * 用法：
 *   useKeyboardShortcuts([
 *     { key: 'Enter', ctrl: true, handler: onSend },
 *     { key: 'l', ctrl: true, handler: onClear },
 *   ])
 */
export function useKeyboardShortcuts(shortcuts: ShortcutConfig[]) {
  const onKeyDown = (e: KeyboardEvent) => {
    for (const s of shortcuts) {
      if (e.key !== s.key) continue
      if (!!s.ctrl !== e.ctrlKey) continue
      if (!!s.shift !== e.shiftKey) continue
      if (!!s.alt !== e.altKey) continue

      // 如果指定了 target，检查焦点是否在目标元素内
      if (s.target) {
        const active = document.activeElement
        if (!active || !s.target.contains(active)) continue
      }

      e.preventDefault()
      s.handler(e)
      break
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', onKeyDown)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', onKeyDown)
  })
}
