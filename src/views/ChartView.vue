<script setup lang="ts">
import { ref } from 'vue'

// 图表相关状态
const isRunning = ref(false)
const chartData = ref<number[]>([])

const toggleChart = () => {
  isRunning.value = !isRunning.value
}

const clearChart = () => {
  chartData.value = []
}
</script>

<template>
  <div class="p-6 h-full flex flex-col">
    <!-- 标题栏 -->
    <div class="mb-6">
      <h2 class="text-3xl font-bold text-gray-800">波形图</h2>
      <p class="text-gray-600 mt-1">实时数据可视化</p>
    </div>

    <!-- 控制面板 -->
    <div class="bg-white rounded-lg shadow-md p-6 mb-6">
      <div class="flex gap-4">
        <button
          @click="toggleChart"
          class="px-6 py-2 rounded-md font-medium transition-colors"
          :class="isRunning 
            ? 'bg-red-500 hover:bg-red-600 text-white' 
            : 'bg-green-500 hover:bg-green-600 text-white'
          "
        >
          {{ isRunning ? '停止' : '开始' }}
        </button>
        <button
          @click="clearChart"
          class="px-6 py-2 bg-gray-200 hover:bg-gray-300 rounded-md font-medium"
        >
          清空
        </button>
      </div>
    </div>

    <!-- 图表区域 -->
    <div class="flex-1 bg-white rounded-lg shadow-md p-6">
      <div class="h-full flex items-center justify-center text-gray-400">
        <div class="text-center">
          <p class="text-lg mb-2">📊 图表区域</p>
          <p class="text-sm">图表功能开发中...</p>
        </div>
      </div>
    </div>
  </div>
</template>
