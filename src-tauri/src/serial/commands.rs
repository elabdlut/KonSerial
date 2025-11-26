/// 串口相关的 Tauri 命令接口
use super::port_manager::{PortManager, PortRuntimeInfo};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;

/// 串口信息（简化版，用于序列化）
#[derive(Debug, Clone, Serialize)]
pub struct SerialPortInfoSimple {
    pub port_name: String,
    pub port_type: String,
}

/// 列出可用串口
#[tauri::command]
pub fn list_serial_ports() -> Result<Vec<String>, String> {
    PortManager::list_ports()
        .map(|ports| {
            ports.into_iter()
                .map(|p| p.port_name)
                .collect()
        })
}

/// 获取串口详细信息
#[tauri::command]
pub fn get_serial_ports_info() -> Result<Vec<SerialPortInfoSimple>, String> {
    PortManager::list_ports()
        .map(|ports| {
            ports.into_iter()
                .map(|p| SerialPortInfoSimple {
                    port_name: p.port_name,
                    port_type: format!("{:?}", p.port_type),
                })
                .collect()
        })
}

/// 打开串口（使用前端传来的配置）
#[tauri::command]
pub async fn open_serial_port(
    manager: State<'_, Arc<Mutex<PortManager>>>,
    port_name: String,
    baud_rate: u32,
) -> Result<(), String> {
    let mgr = manager.lock().await;
    mgr.open(port_name, baud_rate).await
}

/// 关闭串口
#[tauri::command]
pub async fn close_serial_port(
    manager: State<'_, Arc<Mutex<PortManager>>>,
) -> Result<(), String> {
    let mgr = manager.lock().await;
    mgr.close().await
}

/// 获取串口运行时状态
#[tauri::command]
pub async fn get_serial_status(
    manager: State<'_, Arc<Mutex<PortManager>>>,
) -> Result<PortRuntimeInfo, String> {
    let mgr = manager.lock().await;
    Ok(mgr.get_runtime_info().await)
}

/// 发送数据
#[tauri::command]
pub async fn send_serial_data(
    manager: State<'_, Arc<Mutex<PortManager>>>,
    data: Vec<u8>,
) -> Result<usize, String> {
    let mgr = manager.lock().await;
    mgr.send(data).await
}

/// 检查串口是否已连接
#[tauri::command]
pub async fn is_serial_connected(
    manager: State<'_, Arc<Mutex<PortManager>>>,
) -> Result<bool, String> {
    let mgr = manager.lock().await;
    Ok(mgr.is_connected().await)
}
