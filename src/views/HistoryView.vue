<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  NButton, NSpace, NTag, NIcon, NTooltip, NScrollbar, NEmpty, useMessage
} from 'naive-ui'
import {
  TimeOutline, TrashOutline, DownloadOutline, RefreshOutline,
  DocumentTextOutline
} from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/core'
import { t } from '@/stores/i18n'

interface SessionInfo {
  id: number
  connection_id: string
  session_type: string
  port_name: string
  baud_rate: number
  protocol: string
  host: string
  port: number
  started_at: string
  ended_at: string | null
  rx_bytes: number
  tx_bytes: number
}

const formatSessionLabel = (s: SessionInfo) => {
  if (s.session_type === 'serial' || !s.session_type) {
    return s.port_name || 'Serial'
  }
  const proto = (s.protocol || s.session_type).toUpperCase()
  return `${proto} ${s.host || ''}:${s.port || 0}`
}

interface DataRecord {
  id: number
  session_id: number
  direction: string
  data: number[]
  timestamp: string
}

const message = useMessage()
const sessions = ref<SessionInfo[]>([])
const selectedSessionId = ref<number | null>(null)
const records = ref<DataRecord[]>([])
const loadingSessions = ref(false)
const loadingRecords = ref(false)
const hexDisplay = ref(true)

// 分页
const page = ref(1)
const pageSize = ref(100)
const totalRecords = ref(0)

const selectedSession = computed(() =>
  sessions.value.find(s => s.id === selectedSessionId.value) || null
)

const loadSessions = async () => {
  loadingSessions.value = true
  try {
    sessions.value = await invoke<SessionInfo[]>('get_sessions')
    if (selectedSessionId.value && !sessions.value.find(s => s.id === selectedSessionId.value)) {
      selectedSessionId.value = null
      records.value = []
    }
  } catch (e) {
    message.error(t('history.loadFail', String(e)))
  } finally {
    loadingSessions.value = false
  }
}

const loadRecords = async (sessionId: number, pageNum = 1) => {
  loadingRecords.value = true
  try {
    const offset = (pageNum - 1) * pageSize.value
    records.value = await invoke<DataRecord[]>('get_session_data', {
      sessionId,
      direction: null as string | null,
      limit: pageSize.value,
      offset,
    })
    // 简化：用返回条数估计总数（后端 limit 默认 1000，实际应返回 count）
    totalRecords.value = records.value.length < pageSize.value
      ? offset + records.value.length
      : offset + pageSize.value + 1 // 暗示还有更多
  } catch (e) {
    message.error(t('history.recordsFail', String(e)))
  } finally {
    loadingRecords.value = false
  }
}

const prevPage = () => {
  if (page.value > 1) {
    page.value--
    if (selectedSessionId.value) loadRecords(selectedSessionId.value, page.value)
  }
}

const nextPage = () => {
  if (records.value.length === pageSize.value) {
    page.value++
    if (selectedSessionId.value) loadRecords(selectedSessionId.value, page.value)
  }
}

const selectSession = (id: number) => {
  selectedSessionId.value = id
  page.value = 1
  loadRecords(id, 1)
}

const deleteSession = async (id: number, e: Event) => {
  e.stopPropagation()
  try {
    await invoke('delete_session', { sessionId: id })
    message.success(t('history.deleted'))
    await loadSessions()
  } catch (err) {
    message.error(t('history.deleteFail', String(err)))
  }
}

const exportSession = async (id: number, e: Event) => {
  e.stopPropagation()
  let url: string | null = null
  try {
    const csv = await invoke<string>('export_session_csv', { sessionId: id })
    const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' })
    url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `session_${id}.csv`
    a.click()
    message.success(t('history.exported'))
  } catch (err) {
    message.error(t('history.exportFail', String(err)))
  } finally {
    if (url) URL.revokeObjectURL(url)
  }
}

const formatData = (data: number[]) => {
  if (hexDisplay.value) {
    return data.map(b => (b & 0xFF).toString(16).toUpperCase().padStart(2, '0')).join(' ')
  }
  const decoder = new TextDecoder('utf-8', { fatal: false })
  return decoder.decode(new Uint8Array(data))
}

const formatBytes = (n: number) => {
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${(n / 1024 / 1024).toFixed(2)} MB`
}

onMounted(() => {
  loadSessions()
})
</script>

<template>
  <div class="history-page">
    <!-- 左侧会话列表 -->
    <aside class="session-panel">
      <div class="panel-header">
        <div class="section-title">
          <NIcon :component="TimeOutline" size="16" />
          <span>{{ t('history.sessions') }}</span>
        </div>
        <NButton size="tiny" quaternary :loading="loadingSessions" @click="loadSessions">
          <template #icon><NIcon :component="RefreshOutline" /></template>
        </NButton>
      </div>

      <NScrollbar class="session-list">
        <div v-if="sessions.length === 0" class="empty-panel">
          <NEmpty :description="t('history.noSessions')" />
        </div>
        <div
          v-for="s in sessions"
          :key="s.id"
          class="session-card"
          :class="{ active: selectedSessionId === s.id }"
          @click="selectSession(s.id)"
        >
          <div class="session-header">
            <span class="session-port">{{ formatSessionLabel(s) }}</span>
            <NTag v-if="s.session_type === 'serial' || !s.session_type" size="small" :bordered="false">{{ s.baud_rate }}</NTag>
            <NTag v-else size="small" :bordered="false">{{ (s.protocol || s.session_type).toUpperCase() }}</NTag>
          </div>
          <div class="session-meta">
            <span>{{ s.started_at }}</span>
            <span v-if="s.ended_at">~ {{ s.ended_at }}</span>
            <span v-else class="tag-live">{{ t('history.active') }}</span>
            <span v-if="s.session_type !== 'serial' && s.session_type">{{ s.host }}:{{ s.port }}</span>
          </div>
          <div class="session-stats">
            <span class="stat rx">RX {{ formatBytes(s.rx_bytes) }}</span>
            <span class="stat tx">TX {{ formatBytes(s.tx_bytes) }}</span>
          </div>
          <div class="session-actions">
            <NTooltip>
              <template #trigger>
                <NButton size="tiny" quaternary @click.stop="exportSession(s.id, $event)">
                  <template #icon><NIcon :component="DownloadOutline" /></template>
                </NButton>
              </template>
              {{ t('history.exportCsv') }}
            </NTooltip>
            <NTooltip>
              <template #trigger>
                <NButton size="tiny" quaternary type="error" @click.stop="deleteSession(s.id, $event)">
                  <template #icon><NIcon :component="TrashOutline" /></template>
                </NButton>
              </template>
              {{ t('history.delete') }}
            </NTooltip>
          </div>
        </div>
      </NScrollbar>
    </aside>

    <!-- 右侧数据详情 -->
    <main class="detail-panel">
      <div class="detail-header">
        <div class="section-title">
          <NIcon :component="DocumentTextOutline" size="16" />
          <span>{{ t('history.records') }}</span>
          <NTag v-if="selectedSession" size="small" type="info">{{ records.length }}</NTag>
        </div>
        <NSpace align="center">
          <NButton
            size="small"
            :type="hexDisplay ? 'primary' : 'default'"
            ghost
            @click="hexDisplay = true"
          >
            HEX
          </NButton>
          <NButton
            size="small"
            :type="!hexDisplay ? 'primary' : 'default'"
            ghost
            @click="hexDisplay = false"
          >
            {{ t('history.text') }}
          </NButton>
        </NSpace>
      </div>

      <div v-if="!selectedSession" class="empty-detail">
        <NEmpty :description="t('history.selectSession')" />
      </div>
      <div v-else class="records-wrapper">
        <div class="records-toolbar">
          <span class="toolbar-info">{{ formatSessionLabel(selectedSession) }}</span>
          <span class="toolbar-info">{{ selectedSession.started_at }}</span>
          <span v-if="selectedSession.session_type !== 'serial' && selectedSession.session_type" class="toolbar-info">{{ (selectedSession.protocol || selectedSession.session_type).toUpperCase() }} {{ selectedSession.host }}:{{ selectedSession.port }}</span>
        </div>
        <NScrollbar class="records-scroll">
          <table class="records-table">
            <thead>
              <tr>
                <th class="col-time">{{ t('history.time') }}</th>
                <th class="col-dir">{{ t('history.direction') }}</th>
                <th class="col-data">{{ t('history.data') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="r in records" :key="r.id" :class="r.direction.toLowerCase()">
                <td class="col-time">{{ r.timestamp }}</td>
                <td class="col-dir">
                  <NTag size="small" :type="r.direction === 'RX' ? 'success' : 'info'">{{ r.direction }}</NTag>
                </td>
                <td class="col-data"><pre>{{ formatData(r.data) }}</pre></td>
              </tr>
            </tbody>
          </table>
          <div v-if="records.length === 0 && !loadingRecords" class="empty-records">
            <NEmpty :description="t('history.noRecords')" />
          </div>
        </NScrollbar>
        <!-- 分页工具栏 -->
        <div class="pagination-bar">
          <NButton size="tiny" :disabled="page <= 1" @click="prevPage">上一页</NButton>
          <span class="page-info">第 {{ page }} 页</span>
          <NButton size="tiny" :disabled="records.length < pageSize" @click="nextPage">下一页</NButton>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.history-page {
  display: flex;
  height: 100%;
  background: var(--bg-page);
  gap: 16px;
  padding: 16px;
}

.session-panel {
  width: 300px;
  flex-shrink: 0;
  background: var(--bg-card);
  border-radius: 12px;
  box-shadow: var(--shadow-card);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.detail-panel {
  flex: 1;
  min-width: 0;
  background: var(--bg-card);
  border-radius: 12px;
  box-shadow: var(--shadow-card);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-header,
.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 16px;
  border-bottom: 1px solid var(--border-color);
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: var(--font-base);
  font-weight: 600;
  color: var(--text-primary);
}

.session-list {
  flex: 1;
  padding: 12px;
}

.empty-panel,
.empty-detail,
.empty-records {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-detail {
  padding: 40px;
}

.session-card {
  background: var(--bg-page);
  border-radius: 10px;
  padding: 12px;
  margin-bottom: 10px;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid transparent;
}
.session-card:hover {
  background: #f5f5f5;
}
.session-card.active {
  border-color: #4098fc;
  background: #e3f2fd;
}

.session-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}
.session-port {
  font-weight: 600;
  color: var(--text-primary);
  font-size: var(--font-sm);
}

.session-meta {
  font-size: var(--font-xs);
  color: var(--text-muted);
  margin-bottom: 8px;
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.tag-live {
  color: #4caf50;
  font-weight: 500;
}

.session-stats {
  display: flex;
  gap: 12px;
  margin-bottom: 8px;
}
.stat {
  font-size: var(--font-xs);
  font-family: 'SF Mono', Monaco, monospace;
  padding: 2px 8px;
  border-radius: 4px;
  background: #fff;
}
.stat.rx { color: #388e3c; }
.stat.tx { color: #1976d2; }

.session-actions {
  display: flex;
  justify-content: flex-end;
  gap: 6px;
}

/* 右侧详情 */
.records-wrapper {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.records-toolbar {
  display: flex;
  gap: 16px;
  padding: 10px 16px;
  font-size: var(--font-xs);
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-page);
}

.records-scroll {
  flex: 1;
  min-height: 0;
}

.records-table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--font-sm);
}
.records-table thead {
  position: sticky;
  top: 0;
  z-index: 1;
}
.records-table th {
  background: var(--bg-page);
  text-align: left;
  padding: 10px 12px;
  font-weight: 600;
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border-color);
}
.records-table td {
  padding: 10px 12px;
  border-bottom: 1px solid var(--border-color);
  vertical-align: top;
}
.records-table tr:hover {
  background: #fafafa;
}
.records-table tr.rx td {
  background: #f6fff6;
}
.records-table tr.tx td {
  background: #f6f9ff;
}

.col-time { width: 150px; white-space: nowrap; }
.col-dir { width: 60px; }
.col-data pre {
  margin: 0;
  font-family: 'SF Mono', Monaco, monospace;
  font-size: var(--font-xs);
  word-break: break-all;
  white-space: pre-wrap;
  color: var(--text-primary);
}

.pagination-bar {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 8px 16px;
  border-top: 1px solid var(--border-color);
  background: var(--bg-page);
}
.page-info {
  font-size: var(--font-xs);
  color: var(--text-secondary);
  min-width: 60px;
  text-align: center;
}
</style>
