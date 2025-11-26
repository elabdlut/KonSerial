/// 串口管理模块
use crate::{log_error, log_info, log_warn};
use serde::{Deserialize, Serialize};
use serialport::{SerialPort, SerialPortInfo};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex, RwLock};

// ========== 运行时状态（后端管理）==========

/// 串口连接状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PortStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// 串口运行时信息（后端维护的真实状态）
#[derive(Debug, Clone, Serialize)]
pub struct PortRuntimeInfo {
    pub status: PortStatus,
    pub connected_port: Option<String>,
    pub current_baud_rate: Option<u32>,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub last_error: Option<String>,
}

/// 串口管理器（后端状态管理）
pub struct PortManager {
    // 当前打开的串口
    port: Arc<Mutex<Option<Box<dyn SerialPort>>>>,
    
    // 运行时状态
    runtime_info: Arc<RwLock<PortRuntimeInfo>>,
    
    // 数据通道
    data_sender: mpsc::UnboundedSender<Vec<u8>>,
}

impl PortManager {
    pub fn new() -> Self {
        let (tx, _rx) = mpsc::unbounded_channel();
        
        Self {
            port: Arc::new(Mutex::new(None)),
            runtime_info: Arc::new(RwLock::new(PortRuntimeInfo {
                status: PortStatus::Disconnected,
                connected_port: None,
                current_baud_rate: None,
                bytes_received: 0,
                bytes_sent: 0,
                last_error: None,
            })),
            data_sender: tx,
        }
    }
    
    /// 枚举可用串口
    pub fn list_ports() -> Result<Vec<SerialPortInfo>, String> {
        serialport::available_ports()
            .map_err(|e| e.to_string())
    }
    
    /// 打开串口（接收前端传来的配置参数）
    pub async fn open(
        &self,
        port_name: String,
        baud_rate: u32,
    ) -> Result<(), String> {
        log_info!(&format!("正在打开串口: {} @ {}", port_name, baud_rate));
        
        // 更新状态为连接中
        {
            let mut info = self.runtime_info.write().await;
            info.status = PortStatus::Connecting;
        }
        
        // 打开串口
        match serialport::new(&port_name, baud_rate)
            .timeout(Duration::from_millis(100))
            .open()
        {
            Ok(port) => {
                *self.port.lock().await = Some(port);
                
                // 更新运行时状态
                let mut info = self.runtime_info.write().await;
                info.status = PortStatus::Connected;
                info.connected_port = Some(port_name);
                info.current_baud_rate = Some(baud_rate);
                info.last_error = None;
                
                log_info!("串口打开成功");
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                
                // 更新错误状态
                let mut info = self.runtime_info.write().await;
                info.status = PortStatus::Error(error_msg.clone());
                info.last_error = Some(error_msg.clone());
                
                log_error!(&format!("打开串口失败: {}", error_msg));
                Err(error_msg)
            }
        }
    }
    
    /// 关闭串口
    pub async fn close(&self) -> Result<(), String> {
        *self.port.lock().await = None;
        
        let mut info = self.runtime_info.write().await;
        info.status = PortStatus::Disconnected;
        info.connected_port = None;
        info.current_baud_rate = None;
        
        log_info!("串口已关闭");
        Ok(())
    }
    
    /// 获取运行时状态（供前端查询）
    pub async fn get_runtime_info(&self) -> PortRuntimeInfo {
        self.runtime_info.read().await.clone()
    }
    
    /// 发送数据
    pub async fn send(&self, data: Vec<u8>) -> Result<usize, String> {
        let mut port_guard = self.port.lock().await;
        
        if let Some(port) = port_guard.as_mut() {
            match port.write(&data) {
                Ok(bytes) => {
                    // 更新统计
                    let mut info = self.runtime_info.write().await;
                    info.bytes_sent += bytes as u64;
                    
                    Ok(bytes)
                }
                Err(e) => Err(e.to_string()),
            }
        } else {
            Err("串口未连接".to_string())
        }
    }
    
    /// 检查是否已连接
    pub async fn is_connected(&self) -> bool {
        let info = self.runtime_info.read().await;
        info.status == PortStatus::Connected
    }
}
