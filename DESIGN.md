# KonSerial - 设计方案

## 项目概述

KonSerial 是一款基于 Tauri + Vue3 和 Rust + TypeScript 构建的现代化、轻量化的串口调试工具。该项目结合了 VOFA+ 和串口调试助手的功能，提供了一个集数据收发、波形显示、多协议支持及自动化脚本于一体的综合调试平台。

## 技术架构

### 前端技术栈
- **框架**: Vue 3 (Composition API) + TypeScript
- **构建工具**: Vite
- **UI组件**: 原生Vue组件，现代化设计
- **状态管理**: Pinia
- **图表库**: ApexCharts + vue3-apexcharts (用于波形显示)
- **样式框架**: Tailwind CSS + PostCSS
- **打包工具**: Tauri

### 后端技术栈
- **语言**: Rust
- **框架**: Tauri
- **串口库**: serialport (Rust crate)
- **网络库**: tokio (异步运行时)
- **JSON处理**: serde_json
- **插件**: tauri-plugin-fs, tauri-plugin-dialog (根据需要选择)

## 项目结构

### 前端项目结构
```
src/
├── assets/                 # 静态资源
│   ├── icons/              # 图标
│   ├── images/             # 图片资源
│   ├── styles/             # 全局样式
│   │   ├── tailwind.css    # Tailwind基础样式导入
│   │   └── globals.css     # 全局自定义样式
│   └── ...
├── components/             # 可复用UI组件
│   ├── common/             # 通用组件 (Button, Input, etc.)
│   │   ├── Button.vue      # 按钮组件 (使用Tailwind CSS)
│   │   ├── Input.vue       # 输入框组件 (使用Tailwind CSS)
│   │   ├── Select.vue      # 选择框组件 (使用Tailwind CSS)
│   │   └── Toggle.vue      # 开关组件 (使用Tailwind CSS)
│   ├── serial/             # 串口相关组件 (PortSelector, BaudRateSelector, etc.)
│   ├── network/            # 网络相关组件 (TCPSocket, UDPSocket, etc.)
│   ├── visualization/      # 可视化组件 (WaveformChart, HexViewer, etc.)
│   │   ├── WaveformChart.vue # ApexCharts波形显示组件 (使用Tailwind CSS)
│   │   ├── ChartControls.vue # 图表控制组件 (缩放、平移等，使用Tailwind CSS)
│   │   └── DataSeriesSelector.vue # 数据系列选择组件 (使用Tailwind CSS)
│   └── script/             # 脚本相关组件 (ScriptEditor, ScriptRunner, etc.)
├── views/                  # 页面级组件
│   ├── HomeView.vue        # 主界面
│   ├── SerialView.vue      # 串口调试页面
│   ├── NetworkView.vue     # 网络调试页面 (TCP/UDP)
│   ├── BluetoothView.vue   # 蓝牙调试页面
│   ├── ScriptView.vue      # 脚本编辑页面
│   ├── SettingsView.vue    # 设置页面
│   └── HistoryView.vue     # 历史记录页面
├── composables/            # Vue组合式函数
│   ├── useSerial.ts        # 串口相关逻辑
│   ├── useNetwork.ts       # 网络相关逻辑
│   ├── useVisualization.ts # 可视化相关逻辑
│   ├── useScript.ts        # 脚本相关逻辑
│   └── useFileSystem.ts    # 文件系统相关逻辑
├── stores/                 # Pinia状态管理
│   ├── serialStore.ts      # 串口状态
│   ├── networkStore.ts     # 网络状态
│   ├── visualizationStore.ts # 可视化状态
│   ├── scriptStore.ts      # 脚本状态
│   └── appStore.ts         # 应用全局状态
├── types/                  # TypeScript类型定义
│   ├── serial.ts           # 串口相关类型
│   ├── network.ts          # 网络相关类型
│   ├── visualization.ts    # 可视化相关类型
│   ├── script.ts           # 脚本相关类型
│   ├── scriptApi.ts        # 脚本API类型定义
│   └── common.ts           # 通用类型
├── utils/                  # 通用工具函数
│   ├── hexUtils.ts         # 十六进制数据处理
│   ├── protocolParser.ts   # 协议解析工具
│   ├── dataLogger.ts       # 数据记录工具
│   ├── chartUtils.ts       # 图表数据处理工具
│   ├── scriptUtils.ts      # 脚本相关工具
│   └── fileHandler.ts      # 文件处理工具
├── plugins/                # Vue插件
├── router/                 # 路由配置
│   └── index.ts
├── App.vue                 # 根组件
├── main.ts                 # 应用入口
└── vite-env.d.ts           # Vite环境类型定义
```

### 后端项目结构
```
src-tauri/
├── src/
│   ├── main.rs             # 应用入口点
│   ├── lib.rs              # 核心功能库
│   ├── serial/             # 串口通信模块
│   │   ├── mod.rs          # 串口模块导出
│   │   ├── port_manager.rs # 串口管理器
│   │   ├── protocol.rs     # 协议处理
│   │   └── data_processor.rs # 数据处理器
│   ├── network/            # 网络通信模块
│   │   ├── mod.rs          # 网络模块导出
│   │   ├── tcp_client.rs   # TCP客户端
│   │   ├── udp_client.rs   # UDP客户端
│   │   └── bluetooth.rs    # 蓝牙通信
│   ├── script/             # 脚本执行模块
│   │   ├── mod.rs          # 脚本模块导出
│   │   ├── engine.rs       # Rhai脚本执行引擎
│   │   ├── context.rs      # 脚本执行上下文
│   │   └── api.rs          # 脚本API接口 (串口操作、网络操作等)
│   ├── data_logger/        # 数据记录模块
│   │   ├── mod.rs          # 数据记录模块导出
│   │   ├── file_logger.rs  # 文件记录器
│   │   └── memory_logger.rs # 内存记录器
│   ├── visualization/      # 可视化数据处理模块
│   │   ├── mod.rs          # 可视化模块导出
│   │   ├── chart_data.rs   # 图表数据处理
│   │   └── waveform.rs     # 波形数据处理
│   └── utils/              # 通用工具
│       ├── mod.rs          # 工具模块导出
│       ├── logger.rs       # 日志工具
│       └── config.rs       # 配置工具
├── Cargo.toml              # Rust依赖管理
├── tauri.conf.json         # Tauri配置
├── capabilities/           # Tauri权限配置
├── icons/                  # 应用图标
└── build.rs                # 构建脚本
```

## 功能实现设计

### 0. 样式和UI设计 (使用Tailwind CSS)
#### 全局样式配置
- 配置Tailwind CSS主题 (颜色、间距、字体等)
- 创建自定义组件类
- 响应式设计断点配置

**tailwind.config.js 示例**:
```js
/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#eff6ff',
          500: '#3b82f6',
          700: '#1d4ed8',
        },
        secondary: {
          50: '#f0f9ff',
          500: '#0ea5e9',
          700: '#0369a1',
        }
      },
      spacing: {
        '18': '4.5rem',
        '88': '22rem',
      }
    },
  },
  plugins: [
    // 可选插件
    require('@tailwindcss/forms'),
    require('@tailwindcss/typography'),
  ],
}
```

**样式导入 (src/assets/styles/tailwind.css)**:
```css
@tailwind base;
@tailwind components;
@tailwind utilities;

/* 自定义样式覆盖 */
@layer components {
  .btn-primary {
    @apply px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors;
  }
  
  .btn-secondary {
    @apply px-4 py-2 bg-gray-200 text-gray-700 rounded hover:bg-gray-300 transition-colors;
  }
  
  .card {
    @apply bg-white rounded-lg shadow-md p-6;
  }
  
  .input-field {
    @apply w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500;
  }
}
```

#### UI组件设计
- 使用Tailwind CSS类创建可重用的UI组件
- 实现响应式布局
- 创建一致的设计语言

**示例组件 (串口配置面板)**:
```vue
<template>
  <div class="card max-w-2xl mx-auto">
    <h2 class="text-2xl font-bold text-gray-800 mb-6">串口配置</h2>
    
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">串口号</label>
        <select class="input-field">
          <option v-for="port in availablePorts" :key="port" :value="port">
            {{ port }}
          </option>
        </select>
      </div>
      
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">波特率</label>
        <select class="input-field">
          <option value="9600">9600</option>
          <option value="19200">19200</option>
          <option value="38400">38400</option>
          <option value="57600">57600</option>
          <option value="115200">115200</option>
        </select>
      </div>
      
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">数据位</label>
        <select class="input-field">
          <option value="8">8</option>
          <option value="7">7</option>
          <option value="6">6</option>
          <option value="5">5</option>
        </select>
      </div>
      
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">停止位</label>
        <select class="input-field">
          <option value="1">1</option>
          <option value="2">2</option>
        </select>
      </div>
    </div>
    
    <div class="mt-6 flex space-x-4 justify-end">
      <button class="btn-secondary px-6 py-2">
        重置
      </button>
      <button class="btn-primary px-6 py-2 bg-green-600 hover:bg-green-700">
        打开串口
      </button>
    </div>
  </div>
</template>
```

### 1. 串口通信功能

#### 实现方式选择
有两种主要的实现方式来集成串口功能：

**方式一：直接使用 serialport crate (推荐)**
- 优点：无需额外的插件依赖，直接使用成熟的 Rust 库
- 优点：完全控制串口逻辑，更灵活
- 优点：你已经安装了 serialport，可以直接使用
- 缺点：需要自己实现 Tauri 命令接口

**方式二：使用 tauri-plugin-serialport 插件**
- 优点：已有完整的插件实现
- 缺点：增加额外依赖，可能不够灵活

#### 推荐实现（直接使用 serialport crate）
这是推荐的实现方式，因为你的项目已经配置了 `serialport` 库，可以直接在 Rust 后端使用。

#### 前端实现
- 使用Tauri命令与后端通信
- 实现串口端口扫描和选择
- 配置波特率、数据位、停止位、校验位等参数
- 数据发送和接收界面
- 集成tauri-plugin-dialog实现文件选择对话框
- 集成tauri-plugin-fs实现数据导出功能

#### 后端实现
- 使用serialport Rust库实现串口通信
- 实现串口的打开、配置、读写、关闭功能
- 提供Tauri命令接口给前端调用
- 实现数据接收的异步监听和事件推送
- 使用tokio处理异步操作和后台任务
- 集成rhai脚本引擎实现自定义脚本功能

**Tokio异步处理实现**:
```rust
use tokio::time::sleep;
use tokio::task;
use std::time::Duration;

// 异步读取串口数据
pub async fn start_serial_reading(
    port_handle: Arc<Mutex<Box<dyn SerialPort>>>,
    window: tauri::Window,
) -> Result<(), Box<dyn std::error::Error>> {
    task::spawn(async move {
        let mut buffer = [0; 1024];
        loop {
            if let Ok(ref mut port) = port_handle.lock() {
                match port.read(&mut buffer) {
                    Ok(n) => {
                        let data = buffer[..n].to_vec();
                        // 发送数据到前端
                        let _ = window.emit("serial_data_received", &data);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        // 超时继续循环
                        continue;
                    }
                    Err(e) => {
                        let _ = window.emit("serial_error", &e.to_string());
                        break;
                    }
                }
            }
            // 小延迟避免过度占用CPU
            sleep(Duration::from_millis(1)).await;
        }
    });
    
    Ok(())
}
```

**Rhai脚本引擎集成**:
```rust
use rhai::{Engine, Scope, AST};

pub struct ScriptEngine {
    engine: Engine,
    scope: Scope<'static>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        
        // 注册串口操作函数到脚本引擎
        engine.register_fn("send_serial", |data: String, port_name: String| {
            // 实现串口发送功能
            println!("通过串口 {} 发送数据: {}", port_name, data);
            true
        });
        
        engine.register_fn("send_hex", |hex_str: String, port_name: String| {
            // 实现十六进制数据发送功能
            println!("通过串口 {} 发送十六进制数据: {}", port_name, hex_str);
            true
        });
        
        engine.register_fn("delay", |ms: i64| {
            // 在脚本中实现延时功能
            std::thread::sleep(Duration::from_millis(ms as u64));
        });
        
        engine.register_fn("log", |msg: String| {
            // 在脚本中记录日志
            println!("脚本日志: {}", msg);
        });
        
        Self {
            engine,
            scope: Scope::new(),
        }
    }
    
    pub fn execute_script(&mut self, script: &str) -> Result<rhai::Dynamic, Box<dyn std::error::Error>> {
        let ast = self.engine.compile(script)?;
        let result = self.engine.run_ast_with_scope(&mut self.scope, &ast)?;
        Ok(result)
    }
}
```

```rust
// 示例串口管理器实现
use serialport::{SerialPort, DataBits, StopBits, Parity};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use tokio::time::sleep;

// 串口配置结构
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct SerialConfig {
    pub port_name: String,
    pub baud_rate: u32,
    pub data_bits: u8, // 5, 6, 7, 8
    pub stop_bits: u8, // 1 or 2
    pub parity: String, // "none", "odd", "even"
}

// 串口管理器
pub struct SerialPortManager {
    port: Option<Box<dyn SerialPort>>,
    is_reading: Arc<Mutex<bool>>,
}

impl SerialPortManager {
    pub fn new() -> Self {
        Self {
            port: None,
            is_reading: Arc::new(Mutex::new(false)),
        }
    }

    pub fn open_port(&mut self, config: SerialConfig) -> Result<(), Box<dyn std::error::Error>> {
        // 配置串口参数
        let mut settings = serialport::new(&config.port_name, config.baud_rate);
        
        settings = settings
            .data_bits(match config.data_bits {
                5 => DataBits::Five,
                6 => DataBits::Six,
                7 => DataBits::Seven,
                8 => DataBits::Eight,
                _ => DataBits::Eight,
            })
            .stop_bits(match config.stop_bits {
                1 => StopBits::One,
                2 => StopBits::Two,
                _ => StopBits::One,
            })
            .parity(match config.parity.as_str() {
                "odd" => Parity::Odd,
                "even" => Parity::Even,
                _ => Parity::None,
            })
            .timeout(Duration::from_millis(10));

        // 打开串口
        let port = settings.open()?;
        self.port = Some(port);
        
        Ok(())
    }
    
    pub fn close_port(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.port = None;
        Ok(())
    }
    
    pub fn write_data(&mut self, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut port) = self.port {
            port.write_all(&data)?;
            Ok(())
        } else {
            Err("串口未打开".into())
        }
    }
    
    // 启动异步读取数据
    pub fn start_reading(&mut self, emit_fn: impl Fn(String, Vec<u8>) + Send + 'static) {
        if let Some(port) = self.port.as_mut() {
            let mut buffer = [0; 1024];
            match port.read(&mut buffer) {
                Ok(n) => {
                    let data = buffer[..n].to_vec();
                    emit_fn("serial_data_received".to_string(), data);
                }
                Err(e) => {
                    eprintln!("串口读取错误: {}", e);
                }
            }
        }
    }
}

// Tauri 命令定义
use tauri::Manager;

#[tauri::command]
pub async fn open_serial_port(
    window: tauri::Window,
    config: SerialConfig,
) -> Result<String, String> {
    // 实现串口打开逻辑
    use std::sync::Mutex;
    use std::collections::HashMap;
    
    // 在实际实现中，这里会维护一个串口管理器实例
    // 我们用一个简化的示例说明概念
    let mut manager = SerialPortManager::new();
    match manager.open_port(config) {
        Ok(_) => {
            // 启动读取线程
            tokio::spawn(async move {
                loop {
                    // 简化示例：实际中这里会异步读取串口数据
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    
                    // 当收到数据时，发送事件到前端
                    // window.emit("serial_data_received", &received_data).unwrap();
                }
            });
            
            Ok("串口打开成功".to_string())
        }
        Err(e) => Err(format!("串口打开失败: {}", e)),
    }
}

#[tauri::command]
pub async fn write_serial_port(
    config: SerialConfig,
    data: Vec<u8>,
) -> Result<String, String> {
    // 实现串口写入逻辑
    // 在实际应用中，这会向已打开的串口写入数据
    Ok("数据发送成功".to_string())
}

#[tauri::command]
pub async fn list_serial_ports() -> Result<Vec<String>, String> {
    // 列出系统中所有可用的串口
    match serialport::available_ports() {
        Ok(ports) => {
            let port_names: Vec<String> = ports.iter()
                .map(|port| port.port_name.clone())
                .collect();
            Ok(port_names)
        }
        Err(e) => Err(format!("获取串口列表失败: {}", e)),
    }
}

#[tauri::command]
pub async fn close_serial_port() -> Result<String, String> {
    // 实现串口关闭逻辑
    Ok("串口已关闭".to_string())
}

#[tauri::command]
pub async fn execute_script(script_content: String) -> Result<String, String> {
    // 执行用户自定义脚本
    let mut script_engine = ScriptEngine::new();
    
    match script_engine.execute_script(&script_content) {
        Ok(_) => Ok("脚本执行成功".to_string()),
        Err(e) => Err(format!("脚本执行错误: {}", e)),
    }
}
```

#### 插件使用策略
对于KonSerial项目，我们采用按需使用Tauri插件的策略：

1. **必需插件**：
   - `tauri-plugin-fs`: 文件系统访问，用于数据保存和配置文件
     - 数据导出功能：将串口接收的数据保存为CSV、JSON、TXT等格式
     - 配置文件管理：保存和读取串口配置、界面设置等
     - 脚本文件操作：加载和保存自定义脚本
   - `tauri-plugin-dialog`: 文件选择对话框
     - 文件发送功能：选择要通过串口发送的文件
     - 数据导出功能：选择导出文件的保存路径
     - 脚本加载功能：选择要执行的脚本文件

2. **可选插件**：
   - `tauri-plugin-serialport`: 如果直接使用serialport crate实现满足不了需求，则考虑使用
   - 其他插件根据具体功能需要添加

3. **自定义实现**：
   - 串口功能：使用serialport crate直接实现（你已安装）
   - 网络功能：使用tokio和标准网络库实现
   - 脚本功能：使用rhai等Rust脚本引擎

### 2. 波形可视化功能
#### 前端实现
- 使用ApexCharts + vue3-apexcharts实现实时波形显示
- 支持多通道数据同时显示
- 提供缩放、平移等交互功能
- 支持多种数据格式解析 (浮点数、整数、协议数据)
- 实现数据点高亮和详细信息显示
- 使用Tailwind CSS进行样式设计，响应式布局

**ApexCharts组件示例 (使用Tailwind CSS)**:
```vue
<template>
  <div class="p-4 bg-white rounded-lg shadow-md">
    <div class="flex justify-between items-center mb-4">
      <h2 class="text-xl font-semibold text-gray-800">串口数据波形</h2>
      <div class="flex space-x-2">
        <button class="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 text-sm">
          缩放
        </button>
        <button class="px-3 py-1 bg-gray-200 text-gray-700 rounded hover:bg-gray-300 text-sm">
          重置
        </button>
      </div>
    </div>
    <div class="waveform-container">
      <apexchart
        type="line"
        height="400"
        :options="chartOptions"
        :series="series"
      ></apexchart>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import VueApexCharts from 'vue3-apexcharts'
import type { ApexOptions } from 'apexcharts'

// 图表配置
const chartOptions = ref<ApexOptions>({
  chart: {
    id: 'waveform-chart',
    type: 'line',
    height: 400,
    animations: {
      enabled: true,
      easing: 'linear',
      dynamicAnimation: {
        speed: 500
      }
    },
    toolbar: {
      show: false  // 使用自定义按钮
    },
    zoom: {
      enabled: true
    }
  },
  dataLabels: {
    enabled: false
  },
  stroke: {
    curve: 'smooth'
  },
  title: {
    text: undefined  // 在外部标题区域显示
  },
  markers: {
    size: 0
  },
  xaxis: {
    type: 'datetime',
    labels: {
      format: 'HH:mm:ss'
    }
  },
  yaxis: {
    title: {
      text: '数值'
    }
  },
  tooltip: {
    x: {
      format: 'HH:mm:ss'
    }
  },
  legend: {
    position: 'top',
    horizontalAlign: 'right'
  }
})

// 数据序列
const series = ref([
  {
    name: '通道 1',
    data: [] as Array<{ x: number, y: number }>
  },
  {
    name: '通道 2',
    data: [] as Array<{ x: number, y: number }>
  }
])

// 更新数据函数
const updateData = (newData: { channel: number, value: number, timestamp: number }[]) => {
  newData.forEach(item => {
    const channelIndex = item.channel - 1
    if (series.value[channelIndex]) {
      series.value[channelIndex].data.push({
        x: item.timestamp,
        y: item.value
      })
      
      // 限制数据点数量以保持性能
      if (series.value[channelIndex].data.length > 1000) {
        series.value[channelIndex].data.shift()
      }
    }
  })
}

// 监听来自后端的串口数据
// onMounted(() => {
//   const unlisten = await invoke('listen_serial_data', (event) => {
//     updateData(event.payload)
//   })
// })
</script>
```

#### 后端实现
- 实现数据解析和格式转换
- 提供波形数据处理功能
- 缓存最近的可视化数据
- 实现数据采样以优化前端显示性能

### 3. 文件发送功能
#### 前端实现
- 文件选择界面
- 文件内容预览
- 发送进度显示
- 文件发送配置 (发送速度、分包大小等)

#### 后端实现
- 文件读取和分块发送
- 支持大文件处理
- 发送进度跟踪

### 4. 自定义脚本功能
#### 前端实现
- 脚本编辑器 (基于Monaco Editor或CodeMirror)
- 脚本执行控制 (启动、暂停、停止)
- 脚本变量管理
- 脚本模板库
- 脚本语法高亮和错误提示

**脚本编辑器组件示例**:
```vue
<template>
  <div class="script-editor-container">
    <div class="script-toolbar">
      <button @click="runScript">运行</button>
      <button @click="stopScript">停止</button>
      <select v-model="selectedTemplate" @change="loadTemplate">
        <option value="">选择模板</option>
        <option value="periodic-send">周期发送</option>
        <option value="conditional-send">条件发送</option>
        <option value="data-process">数据处理</option>
      </select>
    </div>
    <textarea 
      v-model="scriptContent" 
      class="script-editor"
      placeholder="在此编写脚本..."
    ></textarea>
    <div class="script-output">
      <h4>脚本输出:</h4>
      <pre>{{ scriptOutput }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const scriptContent = ref('')
const scriptOutput = ref('')
const selectedTemplate = ref('')

const runScript = async () => {
  try {
    const result = await invoke('execute_script', { scriptContent: scriptContent.value })
    scriptOutput.value = result
  } catch (error) {
    scriptOutput.value = `错误: ${error}`
  }
}

const stopScript = () => {
  // 停止脚本执行的逻辑
  scriptOutput.value = '脚本已停止'
}

const loadTemplate = () => {
  // 根据选择的模板加载示例代码
  switch(selectedTemplate.value) {
    case 'periodic-send':
      scriptContent.value = `// 周期发送示例
for i in 0..10 {
    send_serial("Hello " + i.to_string(), "COM1")
    delay(1000)  // 延时1秒
}`
      break
    case 'conditional-send':
      scriptContent.value = `// 条件发送示例
let received_data = listen_serial()
if received_data.contains("ACK") {
    send_serial("Command OK", "COM1")
}`
      break
    default:
      scriptContent.value = ''
  }
}
</script>
```

#### 后端实现 (使用Rhai引擎)
- 集成Rhai脚本引擎 (安全、快速的Rust嵌入式脚本引擎)
- 提供串口操作API给脚本调用 (send_serial, listen_serial等)
- 实现脚本执行的安全沙箱 (限制系统访问权限)
- 提供延时、日志、数据处理等辅助函数
- 实现脚本执行状态管理 (启动、停止、暂停)

```rust
// 示例脚本引擎接口
use rhai::{Engine, Scope};

pub struct ScriptEngine {
    engine: Engine,
    scope: Scope<'static>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        
        // 注册串口操作函数
        engine.register_fn("send_serial", |data: String| {
            // 发送串口数据
        });
        
        Self {
            engine,
            scope: Scope::new(),
        }
    }
    
    pub fn execute_script(&mut self, script: &str) -> Result<rhai::Dynamic, Box<dyn std::error::Error>> {
        self.engine.eval_with_scope::<rhai::Dynamic>(&mut self.scope, script)
    }
}
```

### 5. 数据保存功能
#### 前端实现
- 数据导出格式选择 (CSV, JSON, TXT)
- 选择导出时间范围
- 导出进度显示
- 集成tauri-plugin-dialog实现文件保存对话框
- 集成tauri-plugin-fs实现文件写入操作

**文件导出功能示例**:
```vue
<template>
  <div class="export-panel">
    <button @click="exportData">导出数据</button>
    <select v-model="exportFormat">
      <option value="csv">CSV</option>
      <option value="json">JSON</option>
      <option value="txt">TXT</option>
    </select>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'

const exportFormat = ref('csv')

const exportData = async () => {
  // 显示保存文件对话框
  const filePath = await save({
    filters: [{
      name: exportFormat.value.toUpperCase(),
      extensions: [exportFormat.value]
    }]
  })
  
  if (filePath) {
    // 请求后端格式化数据
    const formattedData = await invoke('format_serial_data', { 
      format: exportFormat.value,
      filePath 
    })
    
    // 使用tauri-plugin-fs写入文件
    await writeTextFile(filePath, formattedData)
  }
}
</script>
```

#### 后端实现
- 数据格式化和文件写入
- 大数据量分批处理
- 文件路径选择 (使用Tauri dialog插件)
- 集成tauri-plugin-fs进行安全的文件系统访问
- 实现数据缓冲以处理大文件
- 实现Tauri命令用于数据格式化

**Tauri命令实现示例**:
```rust
use tauri::State;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

// 存储串口数据的全局状态
struct DataStore {
    serial_data: Vec<SerialDataPoint>,
}

#[derive(Serialize, Deserialize, Clone)]
struct SerialDataPoint {
    timestamp: u64,
    channel: u8,
    value: f64,
}

#[tauri::command]
pub async fn format_serial_data(
    state: State<'_, Mutex<DataStore>>,
    format: String,
    start_time: Option<u64>,
    end_time: Option<u64>,
) -> Result<String, String> {
    let data = state.lock().unwrap();
    
    // 根据时间范围过滤数据
    let filtered_data: Vec<&SerialDataPoint> = data.serial_data
        .iter()
        .filter(|point| {
            if let (Some(start), Some(end)) = (start_time, end_time) {
                point.timestamp >= start && point.timestamp <= end
            } else {
                true
            }
        })
        .collect();

    match format.as_str() {
        "csv" => {
            let mut output = String::from("timestamp,channel,value\n");
            for point in filtered_data {
                output.push_str(&format!("{},{},{}\n", point.timestamp, point.channel, point.value));
            }
            Ok(output)
        },
        "json" => {
            match serde_json::to_string(&filtered_data) {
                Ok(json_str) => Ok(json_str),
                Err(e) => Err(format!("JSON格式化错误: {}", e)),
            }
        },
        "txt" => {
            let mut output = String::new();
            for point in filtered_data {
                output.push_str(&format!("时间: {}, 通道: {}, 值: {}\n", 
                    point.timestamp, point.channel, point.value));
            }
            Ok(output)
        },
        _ => Err("不支持的导出格式".to_string()),
    }
}
```

### 6. TCP/UDP调试功能
#### 前端实现
- 网络连接参数配置 (IP、端口)
- 连接状态显示
- 数据发送接收界面

#### 后端实现
- 使用tokio实现异步TCP/UDP通信
- 提供连接管理功能
- 实现网络数据的格式化处理

### 7. 蓝牙串口调试
#### 前端实现
- 蓝牙设备扫描和连接
- 配对状态显示

#### 后端实现
- 实现蓝牙通信协议
- 蓝牙设备扫描和连接管理
- 与串口通信统一接口

### 8. 十六进制显示功能
#### 前端实现
- 十六进制和ASCII显示切换
- 十六进制编辑器
- 数据格式转换

#### 后端实现
- 十六进制数据处理
- 二进制数据解析

## 界面设计

### 整体布局
- 顶部导航栏: 包含菜单、设置、帮助等
- 侧边栏: 功能选择 (串口、网络、脚本等)
- 主内容区: 根据选择的功能显示对应界面
- 底部状态栏: 显示连接状态、日志等信息

### 核心界面
1. **串口调试界面**
   - 左侧: 串口参数配置 (端口选择、波特率、数据位、停止位等)
   - 中部: 数据收发区域 (显示发送/接收的数据)
   - 右侧: 快捷按钮 (打开/关闭端口、清除数据、保存等)

2. **波形显示界面**
   - 图表区域: 显示实时波形
   - 控制区域: 缩放、选择通道、时间范围等

3. **脚本编辑界面**
   - 代码编辑器: 语法高亮、自动补全
   - 控制面板: 运行、停止、调试等
   - 输出区域: 脚本执行结果

4. **网络调试界面**
   - 连接参数: IP、端口、协议选择
   - 数据收发: 网络数据发送和接收
   - 连接状态: 当前连接状态

## 内部逻辑设计

### 状态管理
- 使用Pinia进行全局状态管理
- 分离不同功能模块的状态
- 实现状态持久化 (使用Tauri配置API)

### 通信架构
- 前后端通过Tauri命令进行通信
- 使用事件系统传递异步数据 (如串口接收的数据)
- 实现请求-响应模式处理同步操作

### 数据流设计
```
用户交互 -> Vue组件 -> Pinia Store -> Tauri Command -> Rust Backend
     ^                                                   |
     |                                                   v
     +------------------------------------ Tauri Events (异步数据)
```

### 异步处理
- 串口数据接收: 使用Tauri事件系统
- 网络数据接收: 异步处理，防止阻塞UI
- 文件操作: 在后端异步执行，通过事件通知前端

## 数据流设计

### 串口数据流
```
1. 用户操作串口参数配置
   -> 前端发送命令到后端
   -> 后端配置串口参数
   -> 返回配置结果

2. 串口数据发送
   -> 用户在前端输入数据
   -> 前端发送写入命令到后端
   -> 后端写入串口
   -> 返回写入结果

3. 串口数据接收
   -> 后端异步监听串口数据
   -> 收到数据后发送Tauri事件
   -> 前端监听事件并更新UI
   -> 可视化模块处理数据并更新波形
```

### 网络数据流
```
1. TCP/UDP连接建立
   -> 用户输入连接参数
   -> 前端发送连接命令到后端
   -> 后端建立网络连接
   -> 返回连接状态

2. 网络数据收发
   -> 类似串口数据流，通过Tauri命令和事件处理
```

### 脚本执行数据流
```
1. 脚本执行
   -> 用户在前端编辑脚本
   -> 前端发送执行命令到后端
   -> 后端启动脚本引擎
   -> 脚本执行结果通过事件返回前端
```

### 文件处理数据流
```
1. 数据导出
   -> 用户选择导出格式和范围
   -> 前端发送导出命令到后端
   -> 后端处理数据并写入文件
   -> 返回导出结果
```

## 性能优化策略

### 前端性能
- 虚拟滚动处理大量数据
- 使用Web Worker处理复杂计算
- 实现数据分页和缓存
- 优化ApexCharts渲染性能（数据采样、限制显示点数）
- 使用requestAnimationFrame优化实时更新
- 实现数据缓冲区减少DOM操作频率
- 配置Tailwind CSS生产优化（删除未使用的样式类）
- 使用Tailwind CSS JIT模式提升开发体验

### 后端性能
- 使用异步编程处理I/O操作
- 合理使用线程池
- 实现数据缓冲区减少系统调用
- 使用零拷贝技术优化大数据传输

### 内存管理
- 实现数据生命周期管理
- 定期清理过期数据
- 使用智能指针避免内存泄漏

## 安全性设计

### 数据安全
- 实现数据加密存储 (可选)
- 文件访问权限控制
- 防止注入攻击 (脚本执行沙箱)

### 系统安全
- 使用Tauri的安全配置
- 限制文件系统访问权限
- 网络连接验证和加密 (可选)

## 可扩展性设计

### 插件架构
- 设计可插拔的功能模块
- 支持第三方协议扩展
- 模块化的UI组件设计

### 协议扩展
- 设计协议解析器接口
- 支持多种通信协议 (Modbus, CAN, I2C等)
- 配置化协议定义

## Tauri插件使用策略

对于KonSerial项目，我们采用"最小化插件依赖"的设计原则，优先使用成熟稳定的Rust crate实现功能，仅在必要时使用Tauri插件。

### 核心原则
1. **优先使用Rust crate**: 如果Rust生态系统中有成熟、稳定的crate实现所需功能，则优先使用crate而不是插件
2. **减少依赖**: 避免不必要的插件依赖，降低项目复杂度和潜在安全风险
3. **性能优化**: 直接使用Rust crate通常性能更好，因为避免了插件层的额外开销
4. **灵活性**: 直接实现功能可以更好地控制行为逻辑

### 已采用的Rust crates
- `serialport`: 串口通信
- `tokio`: 异步运行时和网络通信
- `serde_json`: JSON数据处理
- `rhai`: 脚本引擎 (待集成)
- `tauri`: 框架核心

### 建议使用的Tauri插件
- `tauri-plugin-fs`: 文件系统访问
- `tauri-plugin-dialog`: 对话框和文件选择
- `tauri-plugin-shell`: 外部命令执行 (如果需要)

### 不推荐使用的插件
- `tauri-plugin-serialport`: 因为已使用`serialport` crate
- 以及其他功能可以用Rust crate更好实现的插件

## 测试策略

### 单元测试
- 前端组件测试 (使用Vitest + Vue Test Utils)
- 后端函数测试 (使用Rust内置测试框架)

### 集成测试
- 端到端测试 (使用Cypress或Playwright)
- 通信协议测试

### 性能测试
- 大数据量处理测试
- 长时间运行稳定性测试