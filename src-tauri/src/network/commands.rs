use crate::network::manager::{NetConnectionConfig, NetworkManager, NetworkGlobalInfo};
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tauri::command]
pub async fn open_network_connection(
    connection_id: String,
    config: NetConnectionConfig,
    app_handle: tauri::AppHandle,
    manager: State<'_, Arc<Mutex<NetworkManager>>>,
) -> Result<(), String> {
    manager.inner().lock().await.open(connection_id, config, app_handle).await
}

#[tauri::command]
pub async fn close_network_connection(
    connection_id: String,
    manager: State<'_, Arc<Mutex<NetworkManager>>>,
) -> Result<(), String> {
    manager.inner().lock().await.close(&connection_id).await
}

#[tauri::command]
pub async fn close_all_network_connections(
    manager: State<'_, Arc<Mutex<NetworkManager>>>,
) -> Result<(), String> {
    manager.inner().lock().await.close_all().await;
    Ok(())
}

#[tauri::command]
pub async fn send_network_data(
    connection_id: String,
    data: Vec<u8>,
    peer_id: Option<String>,
    manager: State<'_, Arc<Mutex<NetworkManager>>>,
) -> Result<usize, String> {
    manager.inner().lock().await.send(&connection_id, data, peer_id.as_deref()).await
}

#[tauri::command]
pub async fn get_network_connection_info(
    connection_id: String,
    manager: State<'_, Arc<Mutex<NetworkManager>>>,
) -> Result<crate::network::manager::NetConnectionInfo, String> {
    manager.inner().lock().await.get_connection_info(&connection_id).await
}

#[tauri::command]
pub async fn get_all_network_connections(
    manager: State<'_, Arc<Mutex<NetworkManager>>>,
) -> Result<Vec<crate::network::manager::NetConnectionInfo>, String> {
    Ok(manager.inner().lock().await.get_all_connections().await)
}

#[tauri::command]
pub async fn get_network_global_info(
    manager: State<'_, Arc<Mutex<NetworkManager>>>,
) -> Result<NetworkGlobalInfo, String> {
    Ok(manager.inner().lock().await.get_global_info().await)
}
