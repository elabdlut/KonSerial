// 声明 utils 模块
#[macro_use]
mod utils;
mod data_logger;
mod network;
mod script;
mod serial;
mod visualization;

use crate::utils::config::{AppConfig, default_config_path};
use crate::utils::logger::{Logger, LoggerConfig};
use crate::serial::port_manager::PortManager;
use crate::data_logger::{DataLogger, default_db_path};
use crate::network::manager::NetworkManager;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    Logger::init(LoggerConfig::default());

    log_info!("应用启动中...");

    // 初始化配置（跨平台动态路径）
    let config_path = default_config_path();
    let _config = AppConfig::init(&config_path);
    
    // 初始化数据日志管理器（SQLite）
    let db_path = default_db_path();
    let data_logger = Arc::new(
        DataLogger::new(&db_path).expect("初始化数据库失败")
    );
    log_info!(&format!("数据库已初始化: {}", db_path.display()));
    
    // 初始化串口管理器（注入 DataLogger）
    let port_manager = Arc::new(Mutex::new(PortManager::new(data_logger.clone())));

    // 初始化网络连接管理器（注入 DataLogger）
    let network_manager = Arc::new(Mutex::new(NetworkManager::new(data_logger.clone())));

    log_info!("应用启动成功");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        // 注册全局状态
        .manage(port_manager)
        .manage(data_logger)
        .manage(network_manager)
        .invoke_handler(tauri::generate_handler![
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
            crate::serial::commands::send_serial_data_with_crc,
            crate::serial::commands::send_serial_file,
            crate::serial::commands::is_serial_connected,
            // 网络调试命令
            crate::network::commands::open_network_connection,
            crate::network::commands::close_network_connection,
            crate::network::commands::close_all_network_connections,
            crate::network::commands::send_network_data,
            crate::network::commands::get_network_connection_info,
            crate::network::commands::get_all_network_connections,
            crate::network::commands::get_network_global_info,
            // 数据日志命令
            crate::data_logger::commands::get_sessions,
            crate::data_logger::commands::get_session_data,
            crate::data_logger::commands::delete_session,
            crate::data_logger::commands::export_session_csv,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
