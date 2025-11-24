// 记录一些配置信息，便于用户退出时保存信息，例如界面主题设置、串口相关设置
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// 串口配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: String,
}

// 界面配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiConfig {
    pub theme: String,
    pub language: String,
    pub window_width: u32,
    pub window_height: u32,
}

// 数据处理配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataConfig {
    pub auto_save: bool,
    pub save_interval: u32,
    pub max_buffer_size: u32,
    pub data_format: String,
}

// 上层结构体
#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub serial: SerialConfig,
    pub ui: UiConfig,
    pub data: DataConfig,
    #[serde(skip)]
    pub config_path: Option<PathBuf>,
}

impl AppConfig {
    /// 创建新配置，指定配置文件路径
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        AppConfig {
            serial: SerialConfig {
                port: String::new(),
                baud_rate: 9600,
                data_bits: 8,
                stop_bits: 1,
                parity: String::from("None"),
            },
            ui: UiConfig {
                theme: String::from("light"),
                language: String::from("zh-CN"),
                window_width: 1024,
                window_height: 768,
            },
            data: DataConfig {
                auto_save: true,
                save_interval: 60,
                max_buffer_size: 10000,
                data_format: String::from("text"),
            },
            config_path: Some(path.as_ref().to_path_buf()),
        }
    }
    
    /// 保存配置到存储的路径
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = &self.config_path {
            let config_str = serde_json::to_string_pretty(self)?;
            fs::write(path, config_str)?;
            Ok(())
        } else {
            Err("配置文件路径未设置".into())
        }
    }
    
    /// 从存储的路径加载配置
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(&path)?;
        let mut config: AppConfig = serde_json::from_str(&config_str)?;
        // 保存配置文件路径
        config.config_path = Some(path.as_ref().to_path_buf());
        Ok(config)
    }
    
    /// 重新加载配置（从已存储的路径）
    pub fn reload(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = &self.config_path {
            let config_str = fs::read_to_string(path)?;
            let loaded: AppConfig = serde_json::from_str(&config_str)?;
            
            // 更新配置，保留路径
            self.serial = loaded.serial;
            self.ui = loaded.ui;
            self.data = loaded.data;
            Ok(())
        } else {
            Err("配置文件路径未设置".into())
        }
    }
    
    /// 获取配置文件路径
    pub fn get_path(&self) -> Option<&PathBuf> {
        self.config_path.as_ref()
    }
}
