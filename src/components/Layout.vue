<script setup lang="ts">
import { ref } from 'vue'
import { RouterLink, RouterView, useRoute } from 'vue-router'

const route = useRoute()

interface MenuItem {
  path: string
  icon: string
  label: string
}

const menuItems = ref<MenuItem[]>([
  { path: '/serial', icon: '🔌', label: '串口调试' },
  { path: '/chart', icon: '📊', label: '波形图' },
  { path: '/script', icon: '📝', label: '脚本编辑' },
  { path: '/settings', icon: '⚙️', label: '设置' }
])
</script>

<template>
  <div class="flex h-screen bg-gray-100">
    <!-- 侧边栏 -->
    <aside class="w-64 bg-white shadow-lg">
      <div class="p-6 border-b">
        <h1 class="text-2xl font-bold text-gray-800">KonSerial</h1>
        <p class="text-sm text-gray-500 mt-1">串口调试工具</p>
      </div>
      
      <nav class="p-4">
        <RouterLink
          v-for="item in menuItems"
          :key="item.path"
          :to="item.path"
          class="flex items-center px-4 py-3 mb-2 rounded-lg transition-colors"
          :class="
            route.path === item.path
              ? 'bg-blue-500 text-white'
              : 'text-gray-700 hover:bg-gray-100'
          "
        >
          <span class="text-xl mr-3">{{ item.icon }}</span>
          <span class="font-medium">{{ item.label }}</span>
        </RouterLink>
      </nav>
    </aside>

    <!-- 主内容区 -->
    <main class="flex-1 overflow-auto">
      <RouterView />
    </main>
  </div>
</template>
