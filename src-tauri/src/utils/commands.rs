use crate::utils::config::{AppConfig, default_config_path};

/// 加载配置
#[tauri::command]
pub fn load_config(path: Option<String>) -> Result<AppConfig, String> {
    let config_path = path.unwrap_or_else(|| {
        default_config_path().to_string_lossy().to_string()
    });
    
    AppConfig::load(&config_path).map_err(|e| e.to_string())
}

/// 保存配置
#[tauri::command]
pub fn save_config(config: AppConfig, path: Option<String>) -> Result<(), String> {
    let config_path = path.unwrap_or_else(|| {
        default_config_path().to_string_lossy().to_string()
    });
    
    let mut config = config;
    config.config_path = Some(std::path::PathBuf::from(config_path));
    config.save().map_err(|e| e.to_string())
}

/// 获取默认配置路径
#[tauri::command]
pub fn get_config_path() -> String {
    default_config_path().to_string_lossy().to_string()
}

