<script setup lang="ts">
import Layout from './components/Layout.vue'
import { NMessageProvider, NConfigProvider, zhCN, dateZhCN } from 'naive-ui'
</script>

<template>
  <NConfigProvider :locale="zhCN" :date-locale="dateZhCN">
    <NMessageProvider>
      <Layout />
    </NMessageProvider>
  </NConfigProvider>
</template>