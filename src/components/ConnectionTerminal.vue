<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import {
  NSelect, NSwitch, NButton, NIcon, NInput,
  NCheckboxGroup, NCheckbox, NTag, NScrollbar
} from 'naive-ui'
import { TrashOutline, SwapHorizontalOutline } from '@vicons/ionicons5'
import { formatHex } from '@/utils/hex'

interface LogEntry {
  type: string
  content: string
  time: string
}

const props = defineProps<{
  logs: LogEntry[]
  title: string
  logCount: number
  emptyHint: string
  clearLabel: string
  searchPlaceholder: string
  encodingOptions: { label: string; value: string }[]
}>()

const emit = defineEmits<{
  (e: 'clear'): void
}>()

const hexDisplay = ref(false)
const encoding = ref('utf-8')
const autoScroll = ref(true)
const logFilterText = ref('')
const logFilterTypes = ref<string[]>(['tx', 'rx', 'system', 'error'])

const filteredTerminalLogs = computed(() => {
  const text = logFilterText.value.trim().toLowerCase()
  return props.logs.filter((item) => {
    if (!logFilterTypes.value.includes(item.type)) return false
    if (!text) return true
    return (
      item.content.toLowerCase().includes(text) ||
      item.time.toLowerCase().includes(text)
    )
  })
})

const scrollbarRef = ref()

watch(() => filteredTerminalLogs.value.length, () => {
  if (autoScroll.value) {
    nextTick(() => {
      scrollbarRef.value?.scrollTo({ top: 999999 })
    })
  }
})
</script>

<template>
  <div class="terminal-section">
    <div class="terminal-header">
      <div class="terminal-title">
        <NIcon :component="SwapHorizontalOutline" size="18" />
        <span>{{ title }}</span>
        <NTag size="small" :bordered="false" type="info">{{ logCount }}</NTag>
      </div>
      <NSpace align="center" :size="8">
        <NSelect
          v-model:value="encoding"
          :options="encodingOptions"
          size="small"
          style="width: 90px"
        />
        <NSwitch v-model:value="hexDisplay" size="small">
          <template #checked>HEX</template>
          <template #unchecked>文本</template>
        </NSwitch>
        <NSwitch v-model:value="autoScroll" size="small">
          <template #checked>滚动</template>
          <template #unchecked>滚动</template>
        </NSwitch>
        <NButton size="small" quaternary @click="emit('clear')">
          <template #icon><NIcon :component="TrashOutline" /></template>
          {{ clearLabel }}
        </NButton>
      </NSpace>
    </div>

    <div class="terminal-filter-bar">
      <NInput
        v-model:value="logFilterText"
        :placeholder="searchPlaceholder"
        size="tiny"
        clearable
        style="width: 140px"
      />
      <NCheckboxGroup v-model:value="logFilterTypes" size="small">
        <NSpace :size="8">
          <NCheckbox value="tx" label="TX" />
          <NCheckbox value="rx" label="RX" />
          <NCheckbox value="system" label="SYS" />
          <NCheckbox value="error" label="ERR" />
        </NSpace>
      </NCheckboxGroup>
      <NTag size="small" :bordered="false" type="info">{{ filteredTerminalLogs.length }} / {{ logCount }}</NTag>
    </div>

    <NScrollbar ref="scrollbarRef" class="terminal-content">
      <div class="terminal-body">
        <div
          v-for="(item, idx) in filteredTerminalLogs"
          :key="idx"
          class="log-line"
          :class="item.type"
        >
          <span class="log-time">{{ item.time }}</span>
          <span class="log-type">
            {{ item.type === 'tx' ? 'TX' : item.type === 'rx' ? 'RX' : item.type === 'system' ? 'SYS' : 'ERR' }}
          </span>
          <span class="log-content">
            {{ hexDisplay ? formatHex(item.content) : item.content }}
          </span>
        </div>
        <div v-if="filteredTerminalLogs.length === 0" class="terminal-empty">
          <NIcon :component="SwapHorizontalOutline" size="40" />
          <p>{{ emptyHint }}</p>
        </div>
      </div>
    </NScrollbar>
  </div>
</template>

<style scoped>
.terminal-section {
  flex: 1;
  background: var(--bg-terminal);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.terminal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--terminal-header-bg);
  border-bottom: 1px solid var(--terminal-border);
}

.terminal-filter-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  background: var(--terminal-filter-bg);
  border-bottom: 1px solid var(--terminal-border);
}

.terminal-title {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--terminal-text);
  font-size: var(--font-base);
  font-weight: 500;
}

.terminal-content {
  flex: 1;
  min-height: 0;
}

.terminal-body {
  padding: 12px 16px;
  min-height: 100%;
}

.log-line {
  display: flex;
  gap: 12px;
  padding: 4px 0;
  font-family: 'SF Mono', Monaco, Consolas, monospace;
  font-size: var(--app-font-size, 13px);
  line-height: 1.5;
}

.log-time {
  color: var(--terminal-text-muted);
  flex-shrink: 0;
}

.log-type {
  width: 32px;
  flex-shrink: 0;
  font-weight: 600;
}

.log-line.tx .log-type { color: var(--terminal-tx-color); }
.log-line.rx .log-type { color: var(--terminal-rx-color); }
.log-line.system .log-type { color: var(--terminal-system-color); }
.log-line.error .log-type { color: var(--terminal-error-color); }

.log-content {
  color: var(--terminal-text);
  word-break: break-all;
}

.terminal-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--terminal-empty-text);
  gap: 12px;
}

.terminal-empty p {
  font-size: var(--font-base);
}
</style>
