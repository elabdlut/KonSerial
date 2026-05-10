/**
 * 格式化字节数为人类可读字符串
 * 如: 1024 → "1.00 KB", 1536 → "1.50 KB"
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  const idx = Math.min(i, sizes.length - 1)
  return `${(bytes / Math.pow(k, idx)).toFixed(2)} ${sizes[idx]}`
}

/**
 * 格式化速率为人类可读字符串
 * 如: 1536 → "1.50 KB/s"
 */
export function formatRate(bytesPerSec: number): string {
  if (bytesPerSec === 0) return '0 B/s'
  const k = 1024
  const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s']
  const i = Math.floor(Math.log(bytesPerSec) / Math.log(k))
  const idx = Math.min(i, sizes.length - 1)
  return `${(bytesPerSec / Math.pow(k, idx)).toFixed(2)} ${sizes[idx]}`
}

/**
 * 格式化连接时长
 * 如: "2025-01-15 10:30:00" → "01:23:45"
 */
export function formatDuration(connectedAt: string | null | undefined): string {
  if (!connectedAt) return '--:--:--'
  const start = new Date(connectedAt.replace(' ', 'T'))
  if (isNaN(start.getTime())) return '--:--:--'
  const diff = Math.floor((Date.now() - start.getTime()) / 1000)
  const h = Math.floor(diff / 3600)
  const m = Math.floor((diff % 3600) / 60)
  const s = diff % 60
  return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
}
