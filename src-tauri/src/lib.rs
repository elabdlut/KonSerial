// 声明 utils 模块
#[macro_use]
mod utils;
use crate::utils::config::AppConfig;
use crate::utils::logger::{Logger, LoggerConfig};

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
    
    let config_path = "/home/sratle/.config/konserial/config.json";
    let config = AppConfig::init(config_path);
    
    log_info!(&format!("当前串口波特率: {}", config.serial.baud_rate));
    log_warn!("应用启动成功");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
