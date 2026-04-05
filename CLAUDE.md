# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

KonSerial is a Tauri 2 desktop application for serial port debugging. It supports multiple simultaneous serial connections, real-time waveform visualization, custom automation scripts, and data logging.

- **Frontend**: Vue 3 (Composition API, `<script setup>`) + TypeScript
- **Backend**: Rust (Tauri 2)
- **Package Manager**: pnpm
- **UI Library**: naive-ui + Tailwind CSS v4
- **Charts**: ApexCharts via `vue3-apexcharts`

## Common Commands

```bash
# Install dependencies
pnpm install

# Develop with hot-reload (runs Vite dev server + Tauri)
pnpm tauri dev

# Run frontend dev server only (port 1420, for UI-only work)
pnpm dev

# Build production frontend (type-checks with vue-tsc then vite build)
pnpm build

# Build the full desktop app for the current platform
pnpm tauri build

# Run Tauri CLI commands directly
pnpm tauri <command>
```

There are no automated tests configured in this project currently.

## Architecture

### Frontend (src/)

- **Entry**: `src/main.ts` mounts the Vue app, registers `naive-ui`, and initializes the serial data listener.
- **Routing**: `src/router/index.ts` defines four views: Serial (`/serial`), Chart (`/chart`), Script (`/script`), Settings (`/settings`).
- **State Management**: Reactive modules in `src/stores/` (not Pinia stores in practice):
  - `serial.ts` — holds global runtime info, available ports, active connections, received data buffer, and Tauri event listeners.
  - `config.ts` — loads/saves `AppConfig` (serial params, UI theme, data settings) via Tauri commands.
  - `settings.ts` — theme and font-size helpers that apply CSS variables to the DOM.
- **Path Alias**: `@/` maps to `src/` (configured in `vite.config.ts` and `tsconfig.json`).

### Backend (src-tauri/src/)

- **Entry**: `main.rs` delegates to `lib.rs::run()`.
- **lib.rs** sets up:
  - logging (`utils/logger.rs`),
  - config loaded from the system config dir,
  - SQLite `DataLogger` (injected into `PortManager`),
  - global Tauri state (`port_manager`, `data_logger`),
  - invoke handlers for serial, config, and data-log commands.

#### Serial Module (`serial/`)

- **`port_manager.rs`** — `PortManager` manages multiple concurrent serial connections keyed by `connection_id`. Each connection spawns a blocking Tokio task (`read_loop`) that reads from the port, logs RX data to SQLite, and emits Tauri events (`serial-data`) to the frontend.
- **`commands.rs`** — Tauri commands exposed to the frontend: list ports, open/close connections, send data, query runtime info.
- **`data_process.rs`** and **`protocol.rs`** — protocol parsing and data processing helpers.

#### Data Logger (`data_logger/`)

SQLite-based persistence using `rusqlite` with WAL mode. Two tables:
- `sessions` — one row per opened connection.
- `serial_data` — individual RX/TX records with BLOB data and timestamps.

Commands exposed: `get_sessions`, `get_session_data`, `delete_session`, `export_session_csv`.

#### Other Modules

- `network/` — placeholders for TCP/UDP support.
- `script/` — Rhai script engine integration for automation.
- `visualization/` — backend chart/waveform data helpers.
- `utils/` — config management, logging macros, and helper commands.

### Data Flow

1. **Open connection**: Frontend calls `open_serial_port(connection_id, config)`.
2. **Backend read loop**: `PortManager::read_loop` runs in a blocking task. On each read:
   - increments an atomic byte counter,
   - inserts RX data into SQLite via `DataLogger::log_rx`,
   - emits a Tauri event `serial-data` with `{ connection_id, data: Vec<u8> }`.
3. **Frontend receive**: `stores/serial.ts` listens for `serial-data`, converts bytes to a display string, and appends it to `receivedBuffer` (shared across views, including ChartView).
4. **Send data**: Frontend calls `send_serial_data(connection_id, data)`; backend writes to the port and logs TX to SQLite.

### Styling

- Tailwind CSS v4 is imported in `src/assets/styles.css` (`@import 'tailwindcss';`).
- Custom CSS variables for theming (`--bg-page`, `--text-primary`, etc.) are toggled via a `.dark` class on `html`.
- Font sizes scale globally via `--app-font-size`.

### Dependencies of Note

- Rust: `serialport` (cross-platform serial), `tokio` (async runtime), `rhai` (scripting), `rusqlite` (SQLite), `chrono`, `dirs`.
- Frontend: `naive-ui`, `vue-router`, `pinia` (available but reactive modules in `src/stores/` are used directly), `@tauri-apps/api` and plugins (`fs`, `dialog`, `clipboard-manager`, `opener`).
