// 记录一些配置信息，便于用户退出时保存信息，例如界面主题设置、串口相关设置
use serde::{Deserialize, Serialize};
use std::fs;
use std::path;
use serde_json;

// 串口配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: string,
}

// 界面配置
#[derive(Serailize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub serial: SerialConfig,
    pub ui: UiConfig,
    pub data: DataConfig,
}
