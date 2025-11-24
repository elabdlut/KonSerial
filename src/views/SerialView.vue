<script setup lang="ts">
import { ref } from 'vue'

// 串口配置
const portName = ref('')
const baudRate = ref('9600')
const dataBits = ref('8')
const stopBits = ref('1')
const parity = ref('None')
const isConnected = ref(false)

// 接收和发送数据
const receivedData = ref<string[]>([])
const sendData = ref('')
const hexMode = ref(false)

// 串口列表
const availablePorts = ref<string[]>([])

// 连接/断开串口
const toggleConnection = () => {
  isConnected.value = !isConnected.value
  if (isConnected.value) {
    receivedData.value.push(`[系统] 已连接到 ${portName.value}`)
  } else {
    receivedData.value.push(`[系统] 已断开连接`)
  }
}

// 发送数据
const handleSend = () => {
  if (!sendData.value.trim()) return
  
  receivedData.value.push(`[发送] ${sendData.value}`)
  sendData.value = ''
}

// 清空接收区
const clearReceived = () => {
  receivedData.value = []
}

// 刷新串口列表
const refreshPorts = () => {
  // TODO: 调用 Tauri 命令获取串口列表
  availablePorts.value = ['/dev/ttyUSB0', '/dev/ttyUSB1', 'COM1', 'COM2']
  if (availablePorts.value.length > 0) {
    portName.value = availablePorts.value[0]
  }
}

// 初始化时刷新串口列表
refreshPorts()
</script>

<template>
  <div class="p-6 h-full flex flex-col">
    <!-- 标题栏 -->
    <div class="mb-6">
      <h2 class="text-3xl font-bold text-gray-800">串口调试</h2>
      <p class="text-gray-600 mt-1">配置并控制串口通信</p>
    </div>

    <!-- 串口配置面板 -->
    <div class="bg-white rounded-lg shadow-md p-6 mb-6">
      <h3 class="text-lg font-semibold mb-4 text-gray-700">连接配置</h3>
      
      <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
        <!-- 串口选择 -->
        <div class="flex flex-col">
          <label class="text-sm font-medium text-gray-700 mb-2">串口</label>
          <div class="flex gap-2">
            <select 
              v-model="portName"
              :disabled="isConnected"
              class="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
            >
              <option v-for="port in availablePorts" :key="port" :value="port">
                {{ port }}
              </option>
            </select>
            <button 
              @click="refreshPorts"
              :disabled="isConnected"
              class="px-3 py-2 bg-gray-200 rounded-md hover:bg-gray-300 disabled:opacity-50"
              title="刷新"
            >
              🔄
            </button>
          </div>
        </div>

        <!-- 波特率 -->
        <div class="flex flex-col">
          <label class="text-sm font-medium text-gray-700 mb-2">波特率</label>
          <select 
            v-model="baudRate"
            :disabled="isConnected"
            class="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
          >
            <option value="9600">9600</option>
            <option value="19200">19200</option>
            <option value="38400">38400</option>
            <option value="57600">57600</option>
            <option value="115200">115200</option>
          </select>
        </div>

        <!-- 数据位 -->
        <div class="flex flex-col">
          <label class="text-sm font-medium text-gray-700 mb-2">数据位</label>
          <select 
            v-model="dataBits"
            :disabled="isConnected"
            class="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
          >
            <option value="5">5</option>
            <option value="6">6</option>
            <option value="7">7</option>
            <option value="8">8</option>
          </select>
        </div>

        <!-- 停止位 -->
        <div class="flex flex-col">
          <label class="text-sm font-medium text-gray-700 mb-2">停止位</label>
          <select 
            v-model="stopBits"
            :disabled="isConnected"
            class="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
          >
            <option value="1">1</option>
            <option value="1.5">1.5</option>
            <option value="2">2</option>
          </select>
        </div>

        <!-- 校验位 -->
        <div class="flex flex-col">
          <label class="text-sm font-medium text-gray-700 mb-2">校验位</label>
          <select 
            v-model="parity"
            :disabled="isConnected"
            class="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
          >
            <option value="None">无</option>
            <option value="Odd">奇校验</option>
            <option value="Even">偶校验</option>
          </select>
        </div>

        <!-- 连接按钮 -->
        <div class="flex flex-col justify-end">
          <button
            @click="toggleConnection"
            class="px-6 py-2 rounded-md font-medium transition-colors"
            :class="isConnected 
              ? 'bg-red-500 hover:bg-red-600 text-white' 
              : 'bg-green-500 hover:bg-green-600 text-white'
            "
          >
            {{ isConnected ? '断开' : '连接' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 数据收发区 -->
    <div class="flex-1 grid grid-cols-1 lg:grid-cols-3 gap-6 min-h-0">
      <!-- 接收区 -->
      <div class="lg:col-span-2 bg-white rounded-lg shadow-md p-6 flex flex-col">
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-semibold text-gray-700">接收数据</h3>
          <div class="flex gap-2">
            <label class="flex items-center gap-2 text-sm">
              <input type="checkbox" v-model="hexMode" class="rounded">
              <span>HEX显示</span>
            </label>
            <button
              @click="clearReceived"
              class="px-3 py-1 bg-gray-200 hover:bg-gray-300 rounded-md text-sm"
            >
              清空
            </button>
          </div>
        </div>
        
        <div class="flex-1 bg-gray-50 border border-gray-300 rounded-md p-4 overflow-auto font-mono text-sm">
          <div v-for="(line, index) in receivedData" :key="index" class="mb-1">
            {{ line }}
          </div>
          <div v-if="receivedData.length === 0" class="text-gray-400 text-center mt-8">
            暂无数据
          </div>
        </div>
      </div>

      <!-- 发送区 -->
      <div class="bg-white rounded-lg shadow-md p-6 flex flex-col">
        <h3 class="text-lg font-semibold text-gray-700 mb-4">发送数据</h3>
        
        <textarea
          v-model="sendData"
          :disabled="!isConnected"
          placeholder="输入要发送的数据..."
          class="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none disabled:bg-gray-100"
        ></textarea>

        <button
          @click="handleSend"
          :disabled="!isConnected || !sendData.trim()"
          class="mt-4 px-6 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-md font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          发送
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 所有样式通过 Tailwind 实现 */
</style>
