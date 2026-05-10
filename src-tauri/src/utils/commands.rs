use crate::utils::config::{AppConfig, default_config_path};
use std::path::PathBuf;

/// 校验并规范化用户提供的配置文件路径
/// 只允许写入应用配置目录内的文件，防止路径遍历攻击
fn sanitize_config_path(user_path: Option<String>) -> Result<PathBuf, String> {
    let base_dir = default_config_path()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let path = match user_path {
        Some(p) => {
            let resolved = PathBuf::from(&p);
            // 禁止路径中包含 .. 或绝对路径
            if p.contains("..") || resolved.is_absolute() {
                return Err("非法的配置文件路径".into());
            }
            let combined = base_dir.join(&resolved);
            // 确保解析后的路径仍在 base_dir 内
            match combined.canonicalize() {
                Ok(canonical) => {
                    let base_canonical = base_dir.canonicalize().unwrap_or_else(|_| base_dir.clone());
                    if !canonical.starts_with(&base_canonical) {
                        return Err("配置文件路径超出允许范围".into());
                    }
                    canonical
                }
                Err(_) => {
                    // 文件尚不存在时，检查逻辑路径
                    if !combined.starts_with(&base_dir) {
                        return Err("配置文件路径超出允许范围".into());
                    }
                    combined
                }
            }
        }
        None => default_config_path(),
    };
    Ok(path)
}

/// 加载配置
#[tauri::command]
pub fn load_config(path: Option<String>) -> Result<AppConfig, String> {
    let config_path = sanitize_config_path(path)?;
    AppConfig::load(&config_path).map_err(|e| e.to_string())
}

/// 保存配置
#[tauri::command]
pub fn save_config(config: AppConfig, path: Option<String>) -> Result<(), String> {
    let config_path = sanitize_config_path(path)?;
    let mut config = config;
    config.config_path = Some(config_path);
    config.save().map_err(|e| e.to_string())
}

/// 获取默认配置路径
#[tauri::command]
pub fn get_config_path() -> String {
    default_config_path().to_string_lossy().to_string()
}

