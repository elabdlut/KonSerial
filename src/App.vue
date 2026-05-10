<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import Layout from './components/Layout.vue'
import { NMessageProvider, NConfigProvider, darkTheme } from 'naive-ui'
import { startSerialDataListener, stopSerialDataListener } from '@/stores/serial'
import { startNetworkDataListener, stopNetworkDataListener } from '@/stores/network'
import { loadConfig } from '@/stores/config'
import {
  isDark, naiveThemeOverrides, naiveLocale, naiveDateLocale
} from '@/stores/settings'

const naiveTheme = computed(() => isDark.value ? darkTheme : null)

onMounted(async () => {
  const ok = await loadConfig()
  if (!ok) {
    console.warn('配置加载失败，使用默认设置')
  }
  startSerialDataListener()
  startNetworkDataListener()
})

onUnmounted(() => {
  stopSerialDataListener()
  stopNetworkDataListener()
})
</script>

<template>
  <NConfigProvider
    :theme="naiveTheme"
    :theme-overrides="naiveThemeOverrides"
    :locale="naiveLocale"
    :date-locale="naiveDateLocale"
  >
    <NMessageProvider>
      <Layout />
    </NMessageProvider>
  </NConfigProvider>
</template>