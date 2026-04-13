//! 配置管理模块
//! 记录一些配置信息，便于用户退出时保存信息，例如界面主题设置、串口相关设置
use crate::{log_error, log_info, log_warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// 获取默认配置文件路径（跨平台）
/// Linux: ~/.config/konserial/config.json
/// macOS: ~/Library/Application Support/konserial/config.json
/// Windows: C:\Users\<User>\AppData\Roaming\konserial\config.json
pub fn default_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    config_dir.join("konserial").join("config.json")
}

/// 快捷命令
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuickCommand {
    pub name: String,
    pub content: String,
    pub is_hex: bool,
    pub append_newline: String,
}

fn default_quick_commands() -> Vec<QuickCommand> {
    Vec::new()
}

/// 串口配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: String,
    pub flow_control: String,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_quick_commands")]
    pub quick_commands: Vec<QuickCommand>,
    #[serde(default)]
    pub auto_reconnect: bool,
    #[serde(default = "default_reconnect_interval")]
    pub reconnect_interval_ms: u64,
    #[serde(default = "default_max_reconnect_attempts")]
    pub max_reconnect_attempts: u32,
}

fn default_timeout() -> u64 {
    100
}

fn default_reconnect_interval() -> u64 {
    1000
}

fn default_max_reconnect_attempts() -> u32 {
    3
}

/// 界面配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiConfig {
    pub theme: String,
    pub language: String,
    pub font_size: u8,
    pub sidebar_width: u32,
    pub window_width: u32,
    pub window_height: u32,
}

/// 数据处理配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataConfig {
    pub auto_save: bool,
    pub save_interval: u32,
    pub max_buffer_size: u32,
    pub data_format: String,
}

fn default_network_config() -> NetworkConfig {
    NetworkConfig {
        protocol: String::from("tcp"),
        host: String::from("127.0.0.1"),
        port: 8080,
        auto_reconnect: false,
        reconnect_interval_ms: 1000,
        max_reconnect_attempts: 3,
        quick_commands: Vec::new(),
    }
}

/// 网络配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkConfig {
    pub protocol: String,
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub auto_reconnect: bool,
    #[serde(default = "default_reconnect_interval")]
    pub reconnect_interval_ms: u64,
    #[serde(default = "default_max_reconnect_attempts")]
    pub max_reconnect_attempts: u32,
    #[serde(default = "default_quick_commands")]
    pub quick_commands: Vec<QuickCommand>,
}

/// 上层结构体
#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub serial: SerialConfig,
    pub ui: UiConfig,
    pub data: DataConfig,
    #[serde(default = "default_network_config")]
    pub network: NetworkConfig,
    #[serde(skip)]
    pub config_path: Option<PathBuf>,
}

impl AppConfig {
    /// 初始化配置，若存在就加载，不存在就创建并保存
    pub fn init<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();

        if path.exists() {
            match Self::load(path) {
                Ok(cfg) => {
                    log_info!("配置文件加载成功");
                    cfg
                }
                Err(e) => {
                    log_error!(&format!("加载失败:{}, 创建新配置",e));

                    let cfg = Self::new(path);
                    if let Err(e) = cfg.save() {
                        log_error!(&format!("新配置创建失败@{}", e));
                    }
                    cfg
                }
            }
        } else {
            log_warn!("配置不存在，尝试创建新配置");
            let cfg = Self::new(path);
            if let Err(e) = cfg.save() {
                log_error!(&format!("新配置创建失败@{}", e));
            }
            cfg
        }
    }

    /// 创建新配置，指定配置文件路径
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        AppConfig {
            serial: SerialConfig {
                port: String::new(),
                baud_rate: 115200,
                data_bits: 8,
                stop_bits: 1,
                parity: String::from("None"),
                flow_control: String::from("None"),
                timeout_ms: 100,
                quick_commands: Vec::new(),
                auto_reconnect: false,
                reconnect_interval_ms: 1000,
                max_reconnect_attempts: 3,
            },
            ui: UiConfig {
                theme: String::from("light"),
                language: String::from("zh-CN"),
                font_size: 12,
                sidebar_width: 200,
                window_width: 1024,
                window_height: 768,
            },
            data: DataConfig {
                auto_save: true,
                save_interval: 60,
                max_buffer_size: 10000,
                data_format: String::from("text"),
            },
            network: default_network_config(),
            config_path: Some(path.as_ref().to_path_buf()),
        }
    }

    /// 保存配置到存储的路径
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = &self.config_path {
            // 自动创建父目录
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            let config_str = serde_json::to_string_pretty(self)?;
            fs::write(path, config_str)?;
            log_info!("配置文件已保存");
            Ok(())
        } else {
            log_error!("配置文件路径未设置");
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
            self.network = loaded.network;
            Ok(())
        } else {
            log_error!("配置文件路径未设置");
            Err("配置文件路径未设置".into())
        }
    }

    /// 获取配置文件路径
    pub fn get_path(&self) -> Option<&PathBuf> {
        self.config_path.as_ref()
    }
}
