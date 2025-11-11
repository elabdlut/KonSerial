# KonSerial - 现代化轻量化串口调试工具

## 项目概述

KonSerial是一款基于 Tauri + Vue3 和 Rust + TypeScript 构建的现代化、轻量化的串口调试工具。该项目结合了VOFA+和串口调试助手的功能，提供了一个集数据收发、波形显示、协议支持及自动化脚本于一体的综合调试平台。

### 技术栈：
- **前端**：Vue 3 (Composition API) + TypeScript
- **后端**：Rust
- **框架**：Tauri
- **构建系统**：Vite + pnpm
- **UI框架**：Vue原生组件，现代化设计
- **串口通信**：serialport Rust库
- **数据可视化**：支持多种通信协议的波形显示

### 架构：
- **前端**：Vue 3组件位于`src/`目录
- **后端**：Rust代码位于`src-tauri/`目录
- **通信**：Tauri命令实现Rust-JS互操作
- **功能**：串口、TCP/UDP、蓝牙串口等多种协议支持

## 项目结构

```
KonSerial/
├── src/                    # Vue 3 前端源码
│   ├── components/         # Vue组件
│   ├── views/             # 页面视图
│   ├── assets/            # 静态资源
│   ├── App.vue            # 主应用组件
│   └── main.ts            # Vue应用入口点
├── src-tauri/             # Rust 后端源码
│   ├── src/
│   │   ├── main.rs        # Rust应用入口点
│   │   └── lib.rs         # 核心应用逻辑
│   ├── Cargo.toml         # Rust依赖和配置
│   ├── tauri.conf.json    # Tauri配置文件
│   ├── icons/             # 应用图标
│   └── target/            # 编译输出目录
├── public/                # 静态公共资源
├── package.json           # Node.js依赖和脚本
├── vite.config.ts         # Vite构建配置
└── README.md              # 项目说明
```

## 功能特性

### 核心功能
- **数据收发**：支持串口数据的发送和接收，支持十六进制发送和显示
- **波形显示**：实时显示串口发来的数据波形图（支持多种通信协议）
- **文件发送**：支持以文件形式发送串口数据
- **自定义脚本**：支持构建自定义串口发送脚本进行自动化发送
- **数据保存**：支持将接收到的串口数据保存到文件中
- **多协议调试**：支持TCP/UDP调试和蓝牙串口调试

### 高级功能
- 跨平台支持（Windows、macOS、Linux）
- 多种通信协议解析
- 数据格式转换（十六进制、ASCII等）
- 定时发送和循环发送
- 数据过滤和搜索
- 会话保存和加载

## 构建和运行

### 环境要求
- Node.js (推荐最新LTS版本)
- Rust (最新稳定版)
- pnpm
- Tauri构建环境

### 开发环境
```bash
# 安装依赖
pnpm install

# 开发模式运行（实时热重载）
pnpm tauri dev
```

### 生产构建
```bash
# 构建生产版本
pnpm tauri build
```

### 替代命令
```bash
# 仅运行前端（用于UI开发）
pnpm dev

# 仅构建前端
pnpm build

# 预览已构建的前端
pnpm preview

# 运行Tauri特定命令
pnpm tauri [command]
```

## 依赖项

### 前端依赖
- `vue`: ^3.5.13 - 渐进式JavaScript框架
- `@tauri-apps/api`: ^2 - Tauri前端API
- `@tauri-apps/plugin-opener`: ^2 - 文件打开插件

### 开发依赖
- `@tauri-apps/cli`: ^2 - Tauri命令行界面
- `@vitejs/plugin-vue`: ^5.2.1 - Vite的Vue插件
- `typescript`: ~5.6.2 - JavaScript的超集，带类型检查
- `vite`: ^6.0.3 - 前端构建工具
- `vue-tsc`: ^2.1.10 - Vue的TypeScript检查工具

### 后端依赖 (Rust)
- `tauri`: ^2 - 构建原生应用的框架
- `serialport`: ^4.8.1 - 跨平台串口库
- `serde`: 序列化/反序列化框架
- `tauri-plugin-opener`: 文件打开功能插件

## 开发规范

### 前端 (TypeScript/Vue)
- 基于TypeScript的类型安全开发
- 使用Vue 3 Composition API
- 单文件组件(SFC)配合`<script setup>`
- Vite提供快速开发和构建
- 严格的TypeScript配置

### 后端 (Rust)
- 使用Tauri命令从前端暴露Rust功能
- 使用`serialport`库进行串口通信
- 利用Rust的安全性和并发性
- 跨平台兼容性

### UI/UX
- 响应式设计，支持深色/浅色模式
- 现代化的CSS样式，包含作用域样式和全局样式
- 清晰直观的串口调试界面
- 实时数据可视化功能

## 应用场景

- 嵌入式设备调试
- 物联网设备通信测试  
- 工业自动化通信验证
- 数据采集和分析
- 硬件开发和测试
- 教学和实验环境

## 测试

项目目前没有显示明确的测试配置，但通常包括：
- 使用Vitest进行Vue组件测试
- 使用标准Rust测试框架进行Rust单元测试
- 串口通信功能的集成测试
- 协议解析功能的验证测试