/**
 * 格式化日志时间戳为 HH:MM:SS
 */
export function formatLogTime(date: Date | number = Date.now()): string {
  const d = typeof date === 'number' ? new Date(date) : date
  return d.toLocaleTimeString('zh-CN', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

/**
 * 格式化完整日期时间
 */
export function formatDateTime(date: Date | number = Date.now()): string {
  const d = typeof date === 'number' ? new Date(date) : date
  return d.toLocaleString('zh-CN', {
    hour12: false,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}
