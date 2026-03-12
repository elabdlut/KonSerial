<script setup lang="ts">
import { computed, onMounted } from 'vue'
import Layout from './components/Layout.vue'
import { NMessageProvider, NConfigProvider, darkTheme } from 'naive-ui'
import { startSerialDataListener } from '@/stores/serial'
import { loadConfig } from '@/stores/config'
import {
  isDark, naiveThemeOverrides, naiveLocale, naiveDateLocale,
  applyThemeToDOM, applyFontSizeToDOM
} from '@/stores/settings'

const naiveTheme = computed(() => isDark.value ? darkTheme : null)

onMounted(async () => {
  await loadConfig()
  applyThemeToDOM()
  applyFontSizeToDOM()
  startSerialDataListener()
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