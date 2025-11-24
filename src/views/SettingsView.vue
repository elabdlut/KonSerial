<script setup lang="ts">
import { ref } from 'vue'

// 设置项
const theme = ref('light')
const language = ref('zh-CN')
const autoSave = ref(true)
const saveInterval = ref(60)

const saveSettings = () => {
  console.log('保存设置')
  // TODO: 保存到配置文件
}
</script>

<template>
  <div class="p-6 h-full overflow-auto">
    <!-- 标题栏 -->
    <div class="mb-6">
      <h2 class="text-3xl font-bold text-gray-800">设置</h2>
      <p class="text-gray-600 mt-1">应用配置与偏好</p>
    </div>

    <!-- 设置面板 -->
    <div class="max-w-3xl">
      <!-- 外观设置 -->
      <div class="bg-white rounded-lg shadow-md p-6 mb-6">
        <h3 class="text-xl font-semibold text-gray-800 mb-4">外观</h3>
        
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <label class="text-gray-700 font-medium">主题</label>
            <select 
              v-model="theme"
              class="px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="light">浅色</option>
              <option value="dark">深色</option>
              <option value="auto">跟随系统</option>
            </select>
          </div>

          <div class="flex items-center justify-between">
            <label class="text-gray-700 font-medium">语言</label>
            <select 
              v-model="language"
              class="px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="zh-CN">简体中文</option>
              <option value="en-US">English</option>
            </select>
          </div>
        </div>
      </div>

      <!-- 数据设置 -->
      <div class="bg-white rounded-lg shadow-md p-6 mb-6">
        <h3 class="text-xl font-semibold text-gray-800 mb-4">数据</h3>
        
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <label class="text-gray-700 font-medium">自动保存</label>
            <input 
              type="checkbox" 
              v-model="autoSave"
              class="w-5 h-5 text-blue-600 rounded focus:ring-2 focus:ring-blue-500"
            >
          </div>

          <div class="flex items-center justify-between">
            <label class="text-gray-700 font-medium">保存间隔（秒）</label>
            <input 
              type="number" 
              v-model="saveInterval"
              :disabled="!autoSave"
              class="w-24 px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
            >
          </div>
        </div>
      </div>

      <!-- 关于 -->
      <div class="bg-white rounded-lg shadow-md p-6 mb-6">
        <h3 class="text-xl font-semibold text-gray-800 mb-4">关于</h3>
        
        <div class="space-y-2 text-gray-700">
          <p><strong>应用名称：</strong>KonSerial</p>
          <p><strong>版本：</strong>0.1.0</p>
          <p><strong>描述：</strong>现代化轻量级串口调试工具</p>
        </div>
      </div>

      <!-- 保存按钮 -->
      <div class="flex justify-end">
        <button
          @click="saveSettings"
          class="px-8 py-3 bg-blue-500 hover:bg-blue-600 text-white rounded-md font-medium text-lg"
        >
          保存设置
        </button>
      </div>
    </div>
  </div>
</template>
