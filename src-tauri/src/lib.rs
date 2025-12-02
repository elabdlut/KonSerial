// 声明 utils 模块
#[macro_use]
mod utils;
mod data_logger;
mod network;
mod script;
mod serial;
mod visualization;

use crate::utils::config::AppConfig;
use crate::utils::logger::{Logger, LoggerConfig};
use crate::serial::port_manager::PortManager;
use std::sync::Arc;
use tokio::sync::Mutex;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    log::info!("Greet 被调用: name={}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    Logger::init(LoggerConfig::default());

    log_info!("应用启动中...");

    // 初始化配置
    let config_path = "/home/sratle/.config/konserial/config.json";
    let _config = AppConfig::init(config_path);
    
    // 初始化串口管理器
    let port_manager = Arc::new(Mutex::new(PortManager::new()));

    log_warn!("应用启动成功");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        // 注册串口管理器为全局状态
        .manage(port_manager)
        .invoke_handler(tauri::generate_handler![
            // 基础命令
            greet,
            // 配置管理命令
            crate::utils::commands::load_config,
            crate::utils::commands::save_config,
            crate::utils::commands::get_config_path,
            // 串口管理命令
            crate::serial::commands::list_serial_ports,
            crate::serial::commands::get_serial_ports_info,
            crate::serial::commands::refresh_serial_ports,
            crate::serial::commands::open_serial_port,
            crate::serial::commands::close_serial_port,
            crate::serial::commands::close_all_serial_ports,
            crate::serial::commands::get_connection_info,
            crate::serial::commands::get_all_connections,
            crate::serial::commands::get_global_runtime_info,
            crate::serial::commands::send_serial_data,
            crate::serial::commands::is_serial_connected,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
