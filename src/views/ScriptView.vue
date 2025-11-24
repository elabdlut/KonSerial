<script setup lang="ts">
import { ref } from 'vue'

const scriptContent = ref('')

const saveScript = () => {
  console.log('保存脚本:', scriptContent.value)
}

const loadScript = () => {
  console.log('加载脚本')
}
</script>

<template>
  <div class="p-6 h-full flex flex-col">
    <!-- 标题栏 -->
    <div class="mb-6">
      <h2 class="text-3xl font-bold text-gray-800">脚本编辑</h2>
      <p class="text-gray-600 mt-1">编写自动化处理脚本</p>
    </div>

    <!-- 工具栏 -->
    <div class="bg-white rounded-lg shadow-md p-4 mb-6">
      <div class="flex gap-4">
        <button
          @click="loadScript"
          class="px-6 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-md font-medium"
        >
          加载脚本
        </button>
        <button
          @click="saveScript"
          class="px-6 py-2 bg-green-500 hover:bg-green-600 text-white rounded-md font-medium"
        >
          保存脚本
        </button>
      </div>
    </div>

    <!-- 编辑器 -->
    <div class="flex-1 bg-white rounded-lg shadow-md p-6">
      <textarea
        v-model="scriptContent"
        placeholder="// 在此编写脚本代码..."
        class="w-full h-full px-4 py-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none font-mono"
      ></textarea>
    </div>
  </div>
</template>
