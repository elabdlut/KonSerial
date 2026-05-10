<script setup lang="ts">
import { onMounted, onUnmounted, watch } from 'vue'

import {
  NButton, NIcon, NTooltip,
} from 'naive-ui'
import {
  CloseOutline, AddOutline
} from '@vicons/ionicons5'
import NetworkConnectionPane from '@/components/NetworkConnectionPane.vue'
import {
  networkTabs,
  activeTabId,
  activeTab,
  addNetworkTab,
  removeNetworkTab,
  linkTabToConnection,
  updateTabName,
  startStatusPolling,
  stopStatusPolling,
  updateGlobalInfo,
  activeConnections,
  getTabConnectionStatus,
  type NetConnectionConfig,
} from '@/stores/network'

onMounted(async () => {
  if (networkTabs.value.length === 0) {
    addNetworkTab()
  }
  startStatusPolling(1000)
  await updateGlobalInfo()
})

onUnmounted(() => {
  stopStatusPolling()
})

watch(activeConnections, (connections) => {
  for (const tab of networkTabs.value) {
    if (!tab.connectionId) continue
    const conn = connections.find(c => c.connection_id === tab.connectionId)
    if (conn) {
      const active = conn.status === 'Connected' || conn.status === 'Listening'
      const proto = conn.config.protocol.toUpperCase()
      updateTabName(tab.id, `${proto}${active ? '' : ' (断)'}`)
    }
  }
}, { deep: true, immediate: true })

const handleAddTab = () => addNetworkTab()

const handleSelectTab = (tabId: string) => {
  activeTabId.value = tabId
}

const handleCloseTab = async (tabId: string) => {
  try {
    await removeNetworkTab(tabId)
  } catch (e) {
    console.error('关闭标签页失败:', e)
  }
}

const handleUpdateConfig = (tabId: string, config: NetConnectionConfig) => {
  const tab = networkTabs.value.find(t => t.id === tabId)
  if (tab) {
    tab.config = { ...config }
    if (!tab.connectionId) {
      tab.name = config.protocol.toUpperCase()
    }
  }
}

const handleConnect = (tabId: string, _config: NetConnectionConfig, connectionId: string) => {
  linkTabToConnection(tabId, connectionId)
}

const handleDisconnect = (tabId: string) => {
  const tab = networkTabs.value.find(t => t.id === tabId)
  if (tab) {
    tab.connectionId = null
    tab.name = tab.config.protocol.toUpperCase()
  }
}
</script>

<template>
  <div class="network-page">
    <!-- 标签页栏 -->
    <div class="tab-bar">
      <div
        v-for="tab in networkTabs"
        :key="tab.id"
        class="tab-item"
        :class="{ active: tab.id === activeTabId }"
        @click="handleSelectTab(tab.id)"
      >
        <span
          class="status-dot"
          :class="['Connected','Listening'].includes(getTabConnectionStatus(tab) as string) ? 'connected' : 'disconnected'"
        />
        <span class="tab-name">{{ tab.name }}</span>
        <NButton
          v-if="networkTabs.length > 1"
          size="tiny"
          quaternary
          class="close-btn"
          @click.stop="handleCloseTab(tab.id)"
        >
          <template #icon>
            <NIcon :component="CloseOutline" size="12" />
          </template>
        </NButton>
      </div>

      <NTooltip>
        <template #trigger>
          <NButton size="small" quaternary class="add-btn" @click="handleAddTab">
            <template #icon>
              <NIcon :component="AddOutline" />
            </template>
          </NButton>
        </template>
        新建网络连接标签页
      </NTooltip>
    </div>

    <!-- 内容区 -->
    <div class="tab-content">
      <NetworkConnectionPane
        v-if="activeTab"
        :key="activeTab.id"
        :tab-id="activeTab.id"
        :connection-id="activeTab.connectionId"
        :config="activeTab.config"
        @update:config="(cfg) => handleUpdateConfig(activeTab!.id, cfg)"
        @connect="(_cfg, connId) => handleConnect(activeTab!.id, _cfg, connId)"
        @disconnect="handleDisconnect(activeTab!.id)"
      />
      <div v-else class="empty-state">
        <p>暂无标签页</p>
        <NButton @click="handleAddTab">新建标签页</NButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
.network-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-page);
  padding: 12px 16px 16px;
  gap: 12px;
}

.tab-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  overflow-x: auto;
  padding-bottom: 4px;
}

.tab-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  background: var(--bg-card);
  border-radius: 8px 8px 0 0;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  font-size: var(--font-sm);
  color: var(--text-secondary);
  transition: all 0.2s;
  min-width: 100px;
  user-select: none;
}

.tab-item:hover {
  background: var(--bg-sidebar);
}

.tab-item.active {
  border-bottom-color: #4098fc;
  color: var(--text-primary);
  font-weight: 500;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-dot.connected {
  background: #4caf50;
}

.status-dot.disconnected {
  background: #bbb;
}

.tab-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.close-btn {
  opacity: 0;
  flex-shrink: 0;
}

.tab-item:hover .close-btn {
  opacity: 1;
}

.add-btn {
  flex-shrink: 0;
}

.tab-content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-muted);
  gap: 16px;
}
</style>
