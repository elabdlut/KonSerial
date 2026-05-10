<script setup lang="ts">
import { ref } from 'vue'
import {
  NSwitch, NSelect, NSpace, NButton, NIcon, NInput
} from 'naive-ui'
import { SendOutline, CloseOutline, DocumentOutline } from '@vicons/ionicons5'
import type { QuickCommand, NewlineType } from '@/stores/config'

const props = defineProps<{
  isConnected: boolean
  sendDisabled: boolean
  hexSend: boolean
  sendText: string
  showNewline: boolean
  showCrc: boolean
  showFile: boolean
  appendNewline?: NewlineType
  selectedCrc?: string
  newlineOptions?: { label: string; value: NewlineType }[]
  crcOptions?: { label: string; value: string }[]
  quickCommands: QuickCommand[]
  placeholderHex: string
  placeholderText: string
  sendLabel: string
  fileLabel: string
  addQuickCmdLabel: string
  cmdNamePlaceholder: string
  cmdContentPlaceholder: string
}>()

const emit = defineEmits<{
  (e: 'update:hexSend', value: boolean): void
  (e: 'update:sendText', value: string): void
  (e: 'update:appendNewline', value: NewlineType): void
  (e: 'update:selectedCrc', value: string): void
  (e: 'send'): void
  (e: 'sendFile'): void
  (e: 'addQuickCommand', name: string, content: string, isHex: boolean): void
  (e: 'removeQuickCommand', index: number): void
  (e: 'quickSend', cmd: QuickCommand): void
}>()

const quickCmdForm = ref({ name: '', content: '', isHex: false })
const showQuickCmdForm = ref(false)

const onAddQuickCommand = () => {
  if (!quickCmdForm.value.name.trim() || !quickCmdForm.value.content.trim()) return
  emit('addQuickCommand', quickCmdForm.value.name.trim(), quickCmdForm.value.content, quickCmdForm.value.isHex)
  quickCmdForm.value = { name: '', content: '', isHex: false }
  showQuickCmdForm.value = false
}
</script>

<template>
  <div class="send-section">
    <div class="send-options">
      <NSpace align="center" :size="12">
        <NSwitch
          :value="hexSend"
          @update:value="emit('update:hexSend', $event)"
          size="small"
        >
          <template #checked>HEX</template>
          <template #unchecked>文本</template>
        </NSwitch>
        <NSelect
          v-if="showNewline && !hexSend"
          :value="appendNewline"
          @update:value="emit('update:appendNewline', $event)"
          :options="newlineOptions"
          size="small"
          style="width: 120px"
        />
        <NSelect
          v-if="showCrc"
          :value="selectedCrc"
          @update:value="emit('update:selectedCrc', $event)"
          :options="crcOptions"
          size="small"
          style="width: 120px"
        />
      </NSpace>
    </div>

    <!-- 快捷命令 -->
    <div class="quick-commands-bar">
      <div v-if="!showQuickCmdForm" class="quick-commands-list">
        <NButton
          v-for="(cmd, idx) in quickCommands"
          :key="idx"
          size="tiny"
          quaternary
          type="info"
          :disabled="sendDisabled"
          @click="emit('quickSend', cmd)"
        >
          {{ cmd.name }}
          <template #icon>
            <NIcon :component="CloseOutline" @click.stop="emit('removeQuickCommand', idx)" />
          </template>
        </NButton>
        <NButton size="tiny" text @click="showQuickCmdForm = true">
          {{ addQuickCmdLabel }}
        </NButton>
      </div>
      <div v-else class="quick-command-form">
        <NInput
          v-model:value="quickCmdForm.name"
          :placeholder="cmdNamePlaceholder"
          size="tiny"
          style="width: 100px"
        />
        <NInput
          v-model:value="quickCmdForm.content"
          :placeholder="cmdContentPlaceholder"
          size="tiny"
          style="flex: 1"
        />
        <NSwitch v-model:value="quickCmdForm.isHex" size="small">
          <template #checked>HEX</template>
          <template #unchecked>TXT</template>
        </NSwitch>
        <NButton size="tiny" @click="onAddQuickCommand">{{ addQuickCmdLabel }}</NButton>
        <NButton size="tiny" text @click="showQuickCmdForm = false">取消</NButton>
      </div>
    </div>

    <div class="send-input">
      <NInput
        :value="sendText"
        @update:value="emit('update:sendText', $event)"
        :disabled="sendDisabled"
        :placeholder="hexSend ? placeholderHex : placeholderText"
        @keydown.enter="emit('send')"
        clearable
      />
      <NButton
        type="primary"
        :disabled="sendDisabled || !sendText.trim()"
        @click="emit('send')"
      >
        <template #icon><NIcon :component="SendOutline" /></template>
        {{ sendLabel }}
      </NButton>
      <NButton
        v-if="showFile"
        :disabled="sendDisabled"
        @click="emit('sendFile')"
      >
        <template #icon><NIcon :component="DocumentOutline" /></template>
        {{ fileLabel }}
      </NButton>
    </div>
  </div>
</template>

<style scoped>
.send-section {
  background: var(--bg-card);
  border-radius: 12px;
  padding: 16px;
  box-shadow: var(--shadow-card);
}

.send-options {
  margin-bottom: 12px;
}

.send-input {
  display: flex;
  gap: 12px;
}

.send-input :deep(.n-input) {
  flex: 1;
}

.quick-commands-bar {
  margin-bottom: 12px;
}

.quick-commands-list {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.quick-command-form {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
