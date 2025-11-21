# Rust 结构体嵌套与配置文件存储读取教程

## 概述

本教程将详细介绍如何在 Rust 中使用结构体嵌套来组织配置数据，并实现完整的配置文件存储和读取功能。我们将以 KonSerial 串口调试工具的配置系统为例进行说明。

## Rust 结构体嵌套

### 什么是结构体嵌套

结构体嵌套是指在一个结构体中包含其他结构体作为字段。这种方式可以更好地组织复杂的数据结构。

### 基本语法

```rust
#[derive(Debug, Serialize, Deserialize)]
struct Address {
    street: String,
    city: String,
    country: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    name: String,
    age: u32,
    address: Address,  // 嵌套另一个结构体
}
```

## 配置结构体设计

### 简单配置结构（当前实现）

```rust
// 当前的简单配置结构
#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig{
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
}
```

### 嵌套配置结构（推荐实现）

```rust
use serde::{Deserialize, Serialize};
use std::fs;

// 串口配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: String,  // "none", "odd", "even"
}

// 网络配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkConfig {
    pub tcp_enabled: bool,
    pub tcp_port: u16,
    pub udp_enabled: bool,
    pub udp_port: u16,
}

// 界面配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UIConfig {
    pub theme: String,  // "light", "dark", "auto"
    pub language: String,
    pub window_width: u32,
    pub window_height: u32,
}

// 数据处理配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataConfig {
    pub auto_save: bool,
    pub save_interval: u64,  // 秒
    pub max_buffer_size: usize,
    pub data_format: String,  // "hex", "ascii", "decimal"
}

// 主配置结构体
#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub serial: SerialConfig,
    pub network: NetworkConfig,
    pub ui: UIConfig,
    pub data: DataConfig,
    pub last_used: String,  // 最后使用的配置时间戳
}
```

## 配置文件的存储和读取

### 1. 实现配置的保存功能

```rust
use std::fs;
use std::path::Path;
use serde_json;

impl AppConfig {
    /// 保存配置到文件
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let config_str = serde_json::to_string_pretty(self)?;
        fs::write(path, config_str)?;
        Ok(())
    }
    
    /// 从文件加载配置
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let config: AppConfig = serde_json::from_str(&config_str)?;
        Ok(config)
    }
    
    /// 检查配置文件是否存在
    pub fn config_exists<P: AsRef<Path>>(path: P) -> bool {
        Path::new(path.as_ref()).exists()
    }
    
    /// 创建默认配置
    pub fn default() -> Self {
        AppConfig {
            serial: SerialConfig {
                port: "/dev/ttyUSB0".to_string(),  // Linux示例
                // port: "COM1".to_string(),      // Windows示例
                baud_rate: 9600,
                data_bits: 8,
                stop_bits: 1,
                parity: "none".to_string(),
            },
            network: NetworkConfig {
                tcp_enabled: false,
                tcp_port: 8080,
                udp_enabled: false,
                udp_port: 8081,
            },
            ui: UIConfig {
                theme: "auto".to_string(),
                language: "zh-CN".to_string(),
                window_width: 800,
                window_height: 600,
            },
            data: DataConfig {
                auto_save: true,
                save_interval: 300,  // 5分钟
                max_buffer_size: 10000,
                data_format: "hex".to_string(),
            },
            last_used: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// 便捷函数：保存配置
pub fn save_config(config: &AppConfig, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    config.save_to_file(path)
}

/// 便捷函数：加载配置
pub fn load_config(path: &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    if AppConfig::config_exists(path) {
        AppConfig::load_from_file(path)
    } else {
        let default_config = AppConfig::default();
        default_config.save_to_file(path)?;
        Ok(default_config)
    }
}
```

### 2. 完整的配置模块实现

```rust
// src-tauri/src/utils/config.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// 串口配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: String,  // "none", "odd", "even"
}

// 网络配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkConfig {
    pub tcp_enabled: bool,
    pub tcp_port: u16,
    pub udp_enabled: bool,
    pub udp_port: u16,
}

// 界面配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UIConfig {
    pub theme: String,  // "light", "dark", "auto"
    pub language: String,
    pub window_width: u32,
    pub window_height: u32,
}

// 数据处理配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataConfig {
    pub auto_save: bool,
    pub save_interval: u64,  // 秒
    pub max_buffer_size: usize,
    pub data_format: String,  // "hex", "ascii", "decimal"
}

// 主配置结构体
#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub serial: SerialConfig,
    pub network: NetworkConfig,
    pub ui: UIConfig,
    pub data: DataConfig,
    pub last_used: String,  // 最后使用的配置时间戳
}

impl AppConfig {
    /// 保存配置到文件
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let config_str = serde_json::to_string_pretty(self)?;
        fs::write(path, config_str)?;
        Ok(())
    }
    
    /// 从文件加载配置
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let config: AppConfig = serde_json::from_str(&config_str)?;
        Ok(config)
    }
    
    /// 检查配置文件是否存在
    pub fn config_exists<P: AsRef<Path>>(path: P) -> bool {
        Path::new(path.as_ref()).exists()
    }
    
    /// 创建默认配置
    pub fn default() -> Self {
        AppConfig {
            serial: SerialConfig {
                port: "/dev/ttyUSB0".to_string(),  // Linux示例
                baud_rate: 9600,
                data_bits: 8,
                stop_bits: 1,
                parity: "none".to_string(),
            },
            network: NetworkConfig {
                tcp_enabled: false,
                tcp_port: 8080,
                udp_enabled: false,
                udp_port: 8081,
            },
            ui: UIConfig {
                theme: "auto".to_string(),
                language: "zh-CN".to_string(),
                window_width: 800,
                window_height: 600,
            },
            data: DataConfig {
                auto_save: true,
                save_interval: 300,  // 5分钟
                max_buffer_size: 10000,
                data_format: "hex".to_string(),
            },
            last_used: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// 便捷函数：保存配置
pub fn save_config(config: &AppConfig, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    config.save_to_file(path)
}

/// 便捷函数：加载配置
pub fn load_config(path: &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    if AppConfig::config_exists(path) {
        AppConfig::load_from_file(path)
    } else {
        let default_config = AppConfig::default();
        default_config.save_to_file(path)?;
        Ok(default_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_save_load() {
        let config = AppConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap();

        // 保存配置
        assert!(config.save_to_file(temp_path).is_ok());

        // 加载配置
        let loaded_config = load_config(temp_path).unwrap();
        assert_eq!(config.serial.baud_rate, loaded_config.serial.baud_rate);
        assert_eq!(config.ui.theme, loaded_config.ui.theme);
    }
}
```

### 3. 使用示例

```rust
// 在其他模块中使用嵌套配置
use crate::utils::config::{AppConfig, load_config, save_config};

pub fn initialize_app() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = "config.json";
    
    // 加载或创建默认配置
    let mut config = load_config(config_path)?;
    
    // 访问嵌套结构体的字段
    println!("当前串口: {}", config.serial.port);
    println!("波特率: {}", config.serial.baud_rate);
    println!("主题: {}", config.ui.theme);
    
    // 修改配置
    config.serial.baud_rate = 115200;
    config.ui.theme = "dark".to_string();
    config.last_used = chrono::Utc::now().to_rfc3339();
    
    // 保存配置
    save_config(&config, config_path)?;
    
    Ok(())
}
```

### 4. 高级用法：配置验证

```rust
impl AppConfig {
    /// 验证配置的有效性
    pub fn validate(&self) -> Result<(), String> {
        // 验证串口配置
        if self.serial.baud_rate == 0 {
            return Err("波特率不能为0".to_string());
        }
        
        if self.serial.data_bits < 5 || self.serial.data_bits > 8 {
            return Err("数据位必须在5-8之间".to_string());
        }
        
        if self.serial.stop_bits < 1 || self.serial.stop_bits > 2 {
            return Err("停止位必须为1或2".to_string());
        }
        
        if !["none", "odd", "even"].contains(&self.serial.parity.as_str()) {
            return Err("校验位必须是 'none', 'odd' 或 'even'".to_string());
        }
        
        // 验证网络配置
        if self.network.tcp_enabled && (self.network.tcp_port == 0 || self.network.tcp_port > 65535) {
            return Err("TCP端口必须在1-65535之间".to_string());
        }
        
        if self.network.udp_enabled && (self.network.udp_port == 0 || self.network.udp_port > 65535) {
            return Err("UDP端口必须在1-65535之间".to_string());
        }
        
        // 验证界面配置
        if !["light", "dark", "auto"].contains(&self.ui.theme.as_str()) {
            return Err("主题必须是 'light', 'dark' 或 'auto'".to_string());
        }
        
        // 验证数据配置
        if self.data.save_interval == 0 {
            return Err("自动保存间隔不能为0".to_string());
        }
        
        if !["hex", "ascii", "decimal"].contains(&self.data.data_format.as_str()) {
            return Err("数据格式必须是 'hex', 'ascii' 或 'decimal'".to_string());
        }
        
        Ok(())
    }
    
    /// 更新串口配置
    pub fn update_serial_config(&mut self, new_config: SerialConfig) -> Result<(), String> {
        let mut updated_config = self.clone();
        updated_config.serial = new_config;
        updated_config.validate()?;
        
        self.serial = updated_config.serial;
        self.last_used = chrono::Utc::now().to_rfc3339();
        
        Ok(())
    }
}
```

## 最佳实践

### 1. 结构体设计原则

- 将相关的配置项分组到子结构体中
- 使用有意义的字段名称
- 为所有结构体实现必要的 trait（如 Serialize, Deserialize, Debug）

### 2. 错误处理

- 对配置数据进行验证
- 提供清晰的错误信息
- 处理文件不存在的情况

### 3. 性能考虑

- 使用 `Clone` trait 以便复制配置
- 避免不必要的配置重载
- 实现配置缓存机制

### 4. 测试

- 为配置读写功能编写单元测试
- 测试配置验证逻辑
- 使用临时文件进行测试

## 总结

通过结构体嵌套，我们可以更清晰地组织复杂的配置数据，并提供更好的类型安全性和代码可维护性。对于 KonSerial 项目，使用嵌套结构体可以将串口、网络、界面、数据等不同类型的配置项组织到各自的相关结构体中，使代码更易于理解和维护。