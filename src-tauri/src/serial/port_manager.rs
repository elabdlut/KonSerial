/// 串口管理模块
use crate::data_logger::DataLogger;
use crate::{log_error, log_info};
use serde::{Deserialize, Serialize};
use serialport::{SerialPort, SerialPortInfo, DataBits, StopBits, Parity, FlowControl};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex, RwLock};

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

/// 串口数据事件载荷（推送给前端）
#[derive(Clone, Serialize)]
pub struct SerialDataPayload {
    pub connection_id: String,
    pub data: Vec<u8>,
}

/// 单个串口连接（内部结构）
struct SerialConnection {
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    info: ConnectionInfo,
    read_task: tokio::task::JoinHandle<()>,
    running: Arc<AtomicBool>,
    bytes_received_counter: Arc<AtomicU64>,
    session_id: i64,
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

impl From<&SerialPortInfo> for PortInfo {
    fn from(p: &SerialPortInfo) -> Self {
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
    }
}

// ========== 串口管理器（多连接）==========

/// 串口管理器（管理多个串口连接）
pub struct PortManager {
    // 所有活跃的串口连接（key: connection_id）
    connections: Arc<RwLock<HashMap<String, SerialConnection>>>,
    
    // 可用串口缓存
    available_ports_cache: Arc<RwLock<Vec<SerialPortInfo>>>,
    
    // 数据日志管理器
    data_logger: Arc<DataLogger>,
}

impl PortManager {
    pub fn new(data_logger: Arc<DataLogger>) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            available_ports_cache: Arc::new(RwLock::new(Vec::new())),
            data_logger,
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
        let port_infos: Vec<PortInfo> = ports.iter().map(PortInfo::from).collect();
        *self.available_ports_cache.write().await = ports;
        Ok(port_infos)
    }
    
    /// 打开新的串口连接
    pub async fn open(
        &self,
        connection_id: String,
        config: SerialPortConfig,
        app_handle: AppHandle,
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
                // 创建数据记录会话
                let session_id = self.data_logger
                    .create_session(&connection_id, &config.port_name, config.baud_rate)
                    .map_err(|e| format!("创建数据记录会话失败: {}", e))?;
                
                // 克隆串口用于读取循环
                let mut read_port = port.try_clone()
                    .map_err(|e| format!("无法克隆串口用于读取: {}", e))?;
                // 读取端使用固定 100ms 超时，确保能及时响应关闭信号
                let _ = read_port.set_timeout(Duration::from_millis(100));
                
                let running = Arc::new(AtomicBool::new(true));
                let bytes_counter = Arc::new(AtomicU64::new(0));
                
                // 启动后台读取任务
                let conn_id = connection_id.clone();
                let running_clone = running.clone();
                let counter_clone = bytes_counter.clone();
                let logger_clone = self.data_logger.clone();
                let read_task = tokio::task::spawn_blocking(move || {
                    Self::read_loop(
                        read_port, conn_id, app_handle,
                        running_clone, counter_clone,
                        logger_clone, session_id,
                    );
                });
                
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
                    read_task,
                    running,
                    bytes_received_counter: bytes_counter,
                    session_id,
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
    
    /// 串口数据读取循环（在独立线程中运行，通过 Tauri 事件推送数据给前端，同时持久化到 SQLite）
    fn read_loop(
        mut port: Box<dyn SerialPort>,
        connection_id: String,
        app_handle: AppHandle,
        running: Arc<AtomicBool>,
        bytes_counter: Arc<AtomicU64>,
        data_logger: Arc<DataLogger>,
        session_id: i64,
    ) {
        let mut buf = [0u8; 1024];
        while running.load(Ordering::Relaxed) {
            match port.read(&mut buf) {
                Ok(n) if n > 0 => {
                    bytes_counter.fetch_add(n as u64, Ordering::Relaxed);
                    let data = buf[..n].to_vec();
                    // 持久化 RX 数据到 SQLite
                    let _ = data_logger.log_rx(session_id, &data);
                    // 推送到前端
                    let _ = app_handle.emit("serial-data", SerialDataPayload {
                        connection_id: connection_id.clone(),
                        data,
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                Err(_) => break,
                _ => continue,
            }
        }
    }
    
    /// 关闭指定串口连接
    pub async fn close(&self, connection_id: &str) -> Result<(), String> {
        let removed = self.connections.write().await.remove(connection_id);
        
        if let Some(conn) = removed {
            conn.running.store(false, Ordering::Relaxed);
            conn.read_task.abort();
            // 结束数据记录会话
            let _ = self.data_logger.end_session(conn.session_id);
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
        for (_, conn) in conns.drain() {
            conn.running.store(false, Ordering::Relaxed);
            conn.read_task.abort();
            let _ = self.data_logger.end_session(conn.session_id);
        }
        log_info!(&format!("已关闭所有串口连接 ({}个)", count));
    }
    
    /// 获取指定连接的状态
    pub async fn get_connection_info(&self, connection_id: &str) -> Result<ConnectionInfo, String> {
        let conns = self.connections.read().await;
        
        conns.get(connection_id)
            .map(|c| {
                let mut info = c.info.clone();
                info.bytes_received = c.bytes_received_counter.load(Ordering::Relaxed);
                info
            })
            .ok_or_else(|| format!("连接 {} 不存在", connection_id))
    }
    
    /// 获取所有连接的状态
    pub async fn get_all_connections(&self) -> Vec<ConnectionInfo> {
        let conns = self.connections.read().await;
        conns.values().map(|c| {
            let mut info = c.info.clone();
            info.bytes_received = c.bytes_received_counter.load(Ordering::Relaxed);
            info
        }).collect()
    }
    
    /// 获取全局运行时信息
    pub async fn get_global_info(&self) -> GlobalRuntimeInfo {
        let active_connections = self.get_all_connections().await;
        let cached_ports = self.available_ports_cache.read().await;
        let available_ports: Vec<PortInfo> = cached_ports.iter().map(PortInfo::from).collect();
        
        GlobalRuntimeInfo {
            total_connections: active_connections.len(),
            available_ports,
            active_connections,
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
                    // 持久化 TX 数据到 SQLite
                    let _ = self.data_logger.log_tx(conn.session_id, &data[..bytes]);
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
    
    /// 发送文件到指定串口（分块写入，避免缓冲区溢出）
    pub async fn send_file(
        &self,
        connection_id: &str,
        data: Vec<u8>,
        chunk_size: usize,
        delay_ms: u64,
    ) -> Result<usize, String> {
        let mut conns = self.connections.write().await;

        if let Some(conn) = conns.get_mut(connection_id) {
            let mut port = conn.port.lock().await;
            let mut total_sent = 0usize;
            let chunk_size = chunk_size.max(1);

            for chunk in data.chunks(chunk_size) {
                match port.write(chunk) {
                    Ok(bytes) => {
                        total_sent += bytes;
                    }
                    Err(e) => {
                        let error_msg = e.to_string();
                        conn.info.status = PortStatus::Error(error_msg.clone());
                        conn.info.last_error = Some(error_msg.clone());
                        return Err(error_msg);
                    }
                }

                if total_sent < data.len() {
                    drop(port);
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    port = conn.port.lock().await;
                }
            }

            conn.info.bytes_sent += total_sent as u64;
            // 持久化 TX 数据到 SQLite（只记录摘要）
            let _ = self.data_logger.log_tx(conn.session_id, &format!("[FILE] {} bytes", total_sent).into_bytes());
            Ok(total_sent)
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
