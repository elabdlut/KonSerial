# KonSerial v0.2.0 - 现代化轻量化串口/网络调试工具

一款基于 Tauri 2 + Vue 3 + Rust 构建的现代化、轻量化的串口与网络调试工具。集数据收发、实时波形显示、多协议网络调试、自动化脚本及数据持久化于一体。

## 功能特性

### 核心功能
- **多串口同时管理**：支持同时打开多个串口，通过标签页切换不同串口连接
- **数据收发**：支持 ASCII 和十六进制格式的发送与接收，带自动换行与 CRC 校验
- **实时波形显示**：解析串口/网络数据并实时绘制多通道波形图，支持缩放与平移
- **文件发送**：支持以文件形式分块发送数据到串口或网络连接
- **自动化脚本**：内置 JavaScript 脚本编辑器（CodeMirror 语法高亮），可编写自定义发送逻辑
- **数据持久化**：自动将收发数据记录到 SQLite 数据库，支持历史会话查询与 CSV 导出
- **多协议网络调试**：支持 TCP 客户端/服务器、UDP 客户端/服务器、WebSocket、MQTT
- **DTR/RTS 流控**：串口连接支持 DTR、RTS 信号线控制
- **实时统计**：显示 TX/RX 字节总量、实时吞吐量（B/s）及连接时长

### 用户体验
- **现代化 UI**：Vue 3 + Naive UI + Tailwind CSS v4，支持浅色/深色主题自动切换
- **标签页式设计**：可同时管理多个串口和网络连接，独立配置互不干扰
- **配置自动记忆**：串口参数、网络默认配置、UI 偏好自动保存，重启后恢复
- **快捷命令**：支持添加常用发送命令，一键快速发送
- **国际化**：支持中文/英文界面切换
- **跨平台**：基于 Tauri 2，支持 Windows、macOS 和 Linux

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端框架 | Vue 3.5 + TypeScript 5.6 |
| UI 组件库 | Naive UI + Tailwind CSS v4 |
| 代码编辑器 | CodeMirror 6 |
| 构建工具 | Vite 6 + pnpm |
| 桌面框架 | Tauri 2 |
| 后端语言 | Rust 2021 |
| 异步运行时 | Tokio |
| 串口库 | serialport 4.8 |
| 数据库 | SQLite (rusqlite) |
| 网络协议 | TCP/UDP/WebSocket (tokio-tungstenite) / MQTT (rumqttc) |
| CRC 算法 | crc 3 |

## 安装与运行

### 环境要求
- Node.js 20+ (推荐最新 LTS)
- Rust 1.80+ (最新稳定版)
- pnpm

### 开发环境搭建
```bash
# 克隆项目
git clone https://github.com/sratle/KonSerial.git
cd KonSerial

# 安装前端依赖
pnpm install

# 启动开发服务器（Vite + Tauri）
pnpm tauri dev
```

### 生产构建
```bash
# 构建为当前平台的桌面应用
pnpm tauri build

# Linux 下构建 deb 包
pnpm tauri build -- --bundles deb

# 构建产物位于 src-tauri/target/release/bundle/
```

## 使用说明

### 多连接管理
1. 点击标签栏旁的「+」按钮创建新的串口或网络标签页
2. 每个标签页可独立配置参数和连接状态
3. 支持同时打开多个串口和网络连接进行数据对比

### 串口调试
1. 在串口标签页选择串口号和波特率（支持 9600 ~ 4,000,000 bps）
2. 配置数据位、停止位、校验位等参数
3. 点击「打开连接」开始通信
4. 可选启用 DTR/RTS 信号控制

### 网络调试
1. 在网络标签页选择协议类型（TCP/UDP/WS/MQTT/TCP Server/UDP Server）
2. 填写主机地址和端口，MQTT/WebSocket 可配置 Topic/Path
3. TCP/UDP Server 模式会自动监听并管理客户端连接

### 数据收发
- 发送区支持 ASCII 和 HEX 两种模式切换
- 串口支持自动追加换行符（`\n` / `\r\n`）和 CRC 校验
- 接收区实时显示，支持按 TX/RX/System/Error 类型过滤
- 支持搜索日志内容

### 波形显示
- 在 Chart 页面选择要监控的连接
- 数据格式：`channel_name:value`（如 `temp:25.6`）
- 支持多通道同时显示、时间轴缩放/平移、自动缩放 Y 轴
- 支持导出 PNG 图片和 CSV 数据

### 脚本功能
- 在 Script 页面使用内置编辑器编写 JavaScript 脚本
- 脚本通过 `new AsyncFunction()` 在沙箱中执行，可调用预置 API
- 支持选择多个连接同时操作
- 运行日志实时显示在右侧面板

### 历史记录
- History 页面查看所有历史会话及其收发统计
- 点击会话查看详细数据记录（支持 HEX/文本切换）
- 支持导出会话为 CSV 文件
- 数据记录采用分页加载，避免大数据量卡顿

## 协议支持

| 协议 | 模式 | 说明 |
|------|------|------|
| UART/RS232/RS485 | 串口 | 通过 serialport 库访问物理串口 |
| TCP | 客户端 / 服务器 | 支持客户端连接和服务器监听 |
| UDP | 客户端 / 服务器 | 支持单播和广播 |
| WebSocket | 客户端 | 支持 ws:// 连接，自动处理 IPv6 地址格式 |
| MQTT | 客户端 | 基于 rumqttc，支持 QoS 0 |

## 项目结构

```
KonSerial/
├── src/                        # Vue 3 前端源码
│   ├── components/             # Vue 组件
│   │   ├── ConnectionTerminal.vue    # 共享终端组件
│   │   ├── ConnectionSendPane.vue    # 共享发送区组件
│   │   ├── Layout.vue                # 侧边栏布局
│   │   ├── SerialConnectionPane.vue  # 串口连接面板
│   │   └── NetworkConnectionPane.vue # 网络连接面板
│   ├── views/                  # 页面视图
│   │   ├── SerialView.vue      # 串口管理页
│   │   ├── NetworkView.vue     # 网络调试页
│   │   ├── ChartView.vue       # 实时波形图
│   │   ├── ScriptView.vue      # 脚本编辑器
│   │   ├── HistoryView.vue     # 历史记录
│   │   └── SettingsView.vue    # 设置
│   ├── stores/                 # 状态管理
│   ├── composables/            # 组合式函数
│   ├── utils/                  # 工具函数
│   └── types/                  # TypeScript 类型
├── src-tauri/                  # Rust 后端源码
│   ├── src/
│   │   ├── serial/             # 串口管理
│   │   ├── network/            # 网络管理（TCP/UDP/WS/MQTT/Server）
│   │   ├── data_logger/        # SQLite 数据持久化
│   │   ├── utils/              # 配置、日志、路径工具
│   │   └── script/             # 脚本引擎（预留）
│   ├── Cargo.toml
│   └── tauri.conf.json
├── docs/                       # 开发文档
├── public/                     # 公共资源
├── package.json
└── vite.config.ts
```

## 开发贡献

欢迎提交 Issue 和 Pull Request 来改进项目功能。

## 许可证

本项目采用 [MIT 许可证](LICENSE)。
