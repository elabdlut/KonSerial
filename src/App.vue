<script setup lang="ts">
import { onMounted } from 'vue'
import Layout from './components/Layout.vue'
import { NMessageProvider, NConfigProvider, zhCN, dateZhCN } from 'naive-ui'
import { startSerialDataListener } from '@/stores/serial'

onMounted(() => {
  startSerialDataListener()
})
</script>

<template>
  <NConfigProvider :locale="zhCN" :date-locale="dateZhCN">
    <NMessageProvider>
      <Layout />
    </NMessageProvider>
  </NConfigProvider>
</template>