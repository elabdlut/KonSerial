/**
 * 将文本编码为十六进制字符串，每字节两位大写十六进制，以空格分隔
 */
export function formatHex(text: string): string {
  const bytes = new TextEncoder().encode(text)
  return Array.from(bytes)
    .map((b) => b.toString(16).toUpperCase().padStart(2, '0'))
    .join(' ')
}

/**
 * 将十六进制字符串解析为字节数组
 * 支持自动清洗空格，并校验长度是否为偶数
 */
export function parseHex(data: string): number[] {
  const cleaned = data.replace(/\s/g, '')
  if (cleaned.length % 2 !== 0) {
    throw new Error('Hex 数据长度必须为偶数')
  }
  return cleaned.match(/.{1,2}/g)?.map((byte) => parseInt(byte, 16)) || []
}
