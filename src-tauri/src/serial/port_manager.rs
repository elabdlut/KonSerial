/// 串口管理模块
use crate::{log_error, log_info, log_warn};
use serde::{Deserialize, Serialize};
use serialport::{SerialPort, SerialPortInfo, DataBits, StopBits, Parity, FlowControl};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex, RwLock};

// ========== 串口配置（完整参数）==========

/// 完整的串口配置参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialPortConfig {
    pub port_name: String,
    pub baud_rate: u32,
    pub data_bits: u8,        // 5, 6, 7, 8
    pub stop_bits: u8,        // 1, 2
    pub parity: String,       // "None", "Odd", "Even"
    pub flow_control: String, // "None", "Software", "Hardware"
    pub timeout_ms: u64,
}

impl SerialPortConfig {
    /// 转换为 serialport 库的配置
    pub fn to_builder(&self) -> serialport::SerialPortBuilder {
        let mut builder = serialport::new(&self.port_name, self.baud_rate)
            .timeout(Duration::from_millis(self.timeout_ms));
        
        // 数据位
        builder = builder.data_bits(match self.data_bits {
            5 => DataBits::Five,
            6 => DataBits::Six,
            7 => DataBits::Seven,
            _ => DataBits::Eight,
        });
        
        // 停止位
        builder = builder.stop_bits(match self.stop_bits {
            2 => StopBits::Two,
            _ => StopBits::One,
        });
        
        // 校验位
        builder = builder.parity(match self.parity.as_str() {
            "Odd" => Parity::Odd,
            "Even" => Parity::Even,
            _ => Parity::None,
        });
        
        // 流控制
        builder = builder.flow_control(match self.flow_control.as_str() {
            "Software" => FlowControl::Software,
            "Hardware" => FlowControl::Hardware,
            _ => FlowControl::None,
        });
        
        builder
    }
}

// ========== 运行时状态（后端管理）==========

/// 串口连接状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PortStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// 单个串口连接的运行时信息
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionInfo {
    pub connection_id: String,
    pub status: PortStatus,
    pub config: SerialPortConfig,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub last_error: Option<String>,
    pub created_at: String,  // 创建时间戳
}

/// 单个串口连接（内部结构）
struct SerialConnection {
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    info: ConnectionInfo,
}

/// 全局运行时信息（所有串口的状态）
#[derive(Debug, Clone, Serialize)]
pub struct GlobalRuntimeInfo {
    pub available_ports: Vec<PortInfo>,           // 所有可用串口（详细信息）
    pub active_connections: Vec<ConnectionInfo>, // 所有活跃连接
    pub total_connections: usize,
}

/// 串口详细信息
#[derive(Debug, Clone, Serialize)]
pub struct PortInfo {
    pub port_name: String,
    pub port_type: String,       // "USB", "PCI", "Bluetooth", "Unknown"
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<String>,
    pub vid: Option<u16>,        // USB Vendor ID
    pub pid: Option<u16>,        // USB Product ID
}

// ========== 串口管理器（多连接）==========

/// 串口管理器（管理多个串口连接）
pub struct PortManager {
    // 所有活跃的串口连接（key: connection_id）
    connections: Arc<RwLock<HashMap<String, SerialConnection>>>,
    
    // 可用串口缓存
    available_ports_cache: Arc<RwLock<Vec<SerialPortInfo>>>,
}

impl PortManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            available_ports_cache: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// 枚举可用串口
    pub fn list_ports() -> Result<Vec<SerialPortInfo>, String> {
        serialport::available_ports()
            .map_err(|e| e.to_string())
    }
    
    /// 刷新可用串口缓存（返回详细信息）
    pub async fn refresh_available_ports(&self) -> Result<Vec<PortInfo>, String> {
        let ports = Self::list_ports()?;
        let port_infos: Vec<PortInfo> = ports.iter().map(|p| {
            let (port_type, manufacturer, product, serial_number, vid, pid) = match &p.port_type {
                serialport::SerialPortType::UsbPort(usb_info) => (
                    "USB".to_string(),
                    usb_info.manufacturer.clone(),
                    usb_info.product.clone(),
                    usb_info.serial_number.clone(),
                    Some(usb_info.vid),
                    Some(usb_info.pid),
                ),
                serialport::SerialPortType::PciPort => (
                    "PCI".to_string(), None, None, None, None, None
                ),
                serialport::SerialPortType::BluetoothPort => (
                    "Bluetooth".to_string(), None, None, None, None, None
                ),
                serialport::SerialPortType::Unknown => (
                    "Unknown".to_string(), None, None, None, None, None
                ),
            };
            PortInfo {
                port_name: p.port_name.clone(),
                port_type,
                manufacturer,
                product,
                serial_number,
                vid,
                pid,
            }
        }).collect();
        
        *self.available_ports_cache.write().await = ports;
        
        Ok(port_infos)
    }
    
    /// 打开新的串口连接
    pub async fn open(
        &self,
        connection_id: String,
        config: SerialPortConfig,
    ) -> Result<(), String> {
        log_info!(&format!("[{}] 正在打开串口: {} @ {} bps", 
            connection_id, config.port_name, config.baud_rate));
        
        // 检查是否已存在
        {
            let conns = self.connections.read().await;
            if conns.contains_key(&connection_id) {
                return Err(format!("连接 {} 已存在", connection_id));
            }
        }
        
        // 尝试打开串口
        match config.to_builder().open() {
            Ok(port) => {
                let connection = SerialConnection {
                    port: Arc::new(Mutex::new(port)),
                    info: ConnectionInfo {
                        connection_id: connection_id.clone(),
                        status: PortStatus::Connected,
                        config: config.clone(),
                        bytes_received: 0,
                        bytes_sent: 0,
                        last_error: None,
                        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    },
                };
                
                self.connections.write().await.insert(connection_id.clone(), connection);
                
                log_info!(&format!("[{}] 串口打开成功", connection_id));
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                log_error!(&format!("[{}] 打开串口失败: {}", connection_id, error_msg));
                Err(error_msg)
            }
        }
    }
    
    /// 关闭指定串口连接
    pub async fn close(&self, connection_id: &str) -> Result<(), String> {
        let removed = self.connections.write().await.remove(connection_id);
        
        if removed.is_some() {
            log_info!(&format!("[{}] 串口已关闭", connection_id));
            Ok(())
        } else {
            Err(format!("连接 {} 不存在", connection_id))
        }
    }
    
    /// 关闭所有连接
    pub async fn close_all(&self) {
        let mut conns = self.connections.write().await;
        let count = conns.len();
        conns.clear();
        log_info!(&format!("已关闭所有串口连接 ({}个)", count));
    }
    
    /// 获取指定连接的状态
    pub async fn get_connection_info(&self, connection_id: &str) -> Result<ConnectionInfo, String> {
        let conns = self.connections.read().await;
        
        conns.get(connection_id)
            .map(|c| c.info.clone())
            .ok_or_else(|| format!("连接 {} 不存在", connection_id))
    }
    
    /// 获取所有连接的状态
    pub async fn get_all_connections(&self) -> Vec<ConnectionInfo> {
        let conns = self.connections.read().await;
        conns.values().map(|c| c.info.clone()).collect()
    }
    
    /// 获取全局运行时信息
    pub async fn get_global_info(&self) -> GlobalRuntimeInfo {
        let active_connections = self.get_all_connections().await;
        let cached_ports = self.available_ports_cache.read().await;
        
        let available_ports: Vec<PortInfo> = cached_ports.iter().map(|p| {
            let (port_type, manufacturer, product, serial_number, vid, pid) = match &p.port_type {
                serialport::SerialPortType::UsbPort(usb_info) => (
                    "USB".to_string(),
                    usb_info.manufacturer.clone(),
                    usb_info.product.clone(),
                    usb_info.serial_number.clone(),
                    Some(usb_info.vid),
                    Some(usb_info.pid),
                ),
                serialport::SerialPortType::PciPort => (
                    "PCI".to_string(), None, None, None, None, None
                ),
                serialport::SerialPortType::BluetoothPort => (
                    "Bluetooth".to_string(), None, None, None, None, None
                ),
                serialport::SerialPortType::Unknown => (
                    "Unknown".to_string(), None, None, None, None, None
                ),
            };
            PortInfo {
                port_name: p.port_name.clone(),
                port_type,
                manufacturer,
                product,
                serial_number,
                vid,
                pid,
            }
        }).collect();
        
        GlobalRuntimeInfo {
            available_ports,
            active_connections: active_connections.clone(),
            total_connections: active_connections.len(),
        }
    }
    
    /// 发送数据到指定串口
    pub async fn send(&self, connection_id: &str, data: Vec<u8>) -> Result<usize, String> {
        let mut conns = self.connections.write().await;
        
        if let Some(conn) = conns.get_mut(connection_id) {
            let mut port = conn.port.lock().await;
            match port.write(&data) {
                Ok(bytes) => {
                    conn.info.bytes_sent += bytes as u64;
                    Ok(bytes)
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    conn.info.status = PortStatus::Error(error_msg.clone());
                    conn.info.last_error = Some(error_msg.clone());
                    Err(error_msg)
                }
            }
        } else {
            Err(format!("连接 {} 不存在", connection_id))
        }
    }
    
    /// 检查指定连接是否存在且已连接
    pub async fn is_connected(&self, connection_id: &str) -> bool {
        let conns = self.connections.read().await;
        conns.get(connection_id)
            .map(|c| c.info.status == PortStatus::Connected)
            .unwrap_or(false)
    }
}
