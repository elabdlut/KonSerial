/// 串口相关的 Tauri 命令接口
use super::port_manager::{PortManager, SerialPortConfig, ConnectionInfo, GlobalRuntimeInfo, PortInfo};
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

/// 串口信息
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

/// 获取串口详细信息（返回稳定格式）
#[tauri::command]
pub fn get_serial_ports_info() -> Result<Vec<SerialPortInfoSimple>, String> {
    PortManager::list_ports()
        .map(|ports| {
            ports.into_iter()
                .map(|p| SerialPortInfoSimple {
                    port_name: p.port_name,
                    port_type: match p.port_type {
                        serialport::SerialPortType::UsbPort(_) => "USB".to_string(),
                        serialport::SerialPortType::PciPort => "PCI".to_string(),
                        serialport::SerialPortType::BluetoothPort => "Bluetooth".to_string(),
                        serialport::SerialPortType::Unknown => "Unknown".to_string(),
                    },
                })
                .collect()
        })
}

/// 刷新可用串口列表（返回详细信息）
#[tauri::command]
pub async fn refresh_serial_ports(
    manager: State<'_, Arc<Mutex<PortManager>>>,
) -> Result<Vec<PortInfo>, String> {
    let mgr = manager.lock().await;
    mgr.refresh_available_ports().await
}

/// 打开串口（使用完整配置）
#[tauri::command]
pub async fn open_serial_port(
    app: AppHandle,
    manager: State<'_, Arc<Mutex<PortManager>>>,
    connection_id: String,
    config: SerialPortConfig,
) -> Result<(), String> {
    let mgr = manager.lock().await;
    mgr.open(connection_id, config, app).await
}

/// 关闭指定串口
#[tauri::command]
pub async fn close_serial_port(
    manager: State<'_, Arc<Mutex<PortManager>>>,
    connection_id: String,
) -> Result<(), String> {
    let mgr = manager.lock().await;
    mgr.close(&connection_id).await
}

/// 关闭所有串口
#[tauri::command]
pub async fn close_all_serial_ports(
    manager: State<'_, Arc<Mutex<PortManager>>>,
) -> Result<(), String> {
    let mgr = manager.lock().await;
    mgr.close_all().await;
    Ok(())
}

/// 获取指定连接的状态
#[tauri::command]
pub async fn get_connection_info(
    manager: State<'_, Arc<Mutex<PortManager>>>,
    connection_id: String,
) -> Result<ConnectionInfo, String> {
    let mgr = manager.lock().await;
    mgr.get_connection_info(&connection_id).await
}

/// 获取所有连接的状态
#[tauri::command]
pub async fn get_all_connections(
    manager: State<'_, Arc<Mutex<PortManager>>>,
) -> Result<Vec<ConnectionInfo>, String> {
    let mgr = manager.lock().await;
    Ok(mgr.get_all_connections().await)
}

/// 获取全局运行时信息
#[tauri::command]
pub async fn get_global_runtime_info(
    manager: State<'_, Arc<Mutex<PortManager>>>,
) -> Result<GlobalRuntimeInfo, String> {
    let mgr = manager.lock().await;
    Ok(mgr.get_global_info().await)
}

/// 发送数据到指定串口
#[tauri::command]
pub async fn send_serial_data(
    manager: State<'_, Arc<Mutex<PortManager>>>,
    connection_id: String,
    data: Vec<u8>,
) -> Result<usize, String> {
    let mgr = manager.lock().await;
    mgr.send(&connection_id, data).await
}

/// 发送数据到指定串口（自动追加 CRC/校验和）
#[tauri::command]
pub async fn send_serial_data_with_crc(
    manager: State<'_, Arc<Mutex<PortManager>>>,
    connection_id: String,
    mut data: Vec<u8>,
    crc_algorithm: String,
) -> Result<usize, String> {
    use super::protocol::{CrcAlgorithm, append_crc};
    let algo = CrcAlgorithm::from_str(&crc_algorithm)
        .ok_or_else(|| format!("不支持的 CRC 算法: {}", crc_algorithm))?;
    append_crc(&mut data, algo);
    let mgr = manager.lock().await;
    mgr.send(&connection_id, data).await
}

/// 发送文件到指定串口
#[tauri::command]
pub async fn send_serial_file(
    manager: State<'_, Arc<Mutex<PortManager>>>,
    connection_id: String,
    data: Vec<u8>,
    chunk_size: Option<usize>,
    delay_ms: Option<u64>,
) -> Result<usize, String> {
    let mgr = manager.lock().await;
    let chunk_size = chunk_size.unwrap_or(256);
    let delay_ms = delay_ms.unwrap_or(5);
    mgr.send_file(&connection_id, data, chunk_size, delay_ms).await
}

/// 检查指定串口是否已连接
#[tauri::command]
pub async fn is_serial_connected(
    manager: State<'_, Arc<Mutex<PortManager>>>,
    connection_id: String,
) -> Result<bool, String> {
    let mgr = manager.lock().await;
    Ok(mgr.is_connected(&connection_id).await)
}

/// 设置串口 DTR 信号
#[tauri::command]
pub async fn set_serial_dtr(
    manager: State<'_, Arc<Mutex<PortManager>>>,
    connection_id: String,
    level: bool,
) -> Result<(), String> {
    let mgr = manager.lock().await;
    mgr.set_dtr(&connection_id, level).await
}

/// 设置串口 RTS 信号
#[tauri::command]
pub async fn set_serial_rts(
    manager: State<'_, Arc<Mutex<PortManager>>>,
    connection_id: String,
    level: bool,
) -> Result<(), String> {
    let mgr = manager.lock().await;
    mgr.set_rts(&connection_id, level).await
}
