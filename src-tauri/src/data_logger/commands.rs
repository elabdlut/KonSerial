//! 数据日志相关的 Tauri 命令接口

use super::{DataLogger, DataRecord, SessionInfo};
use std::sync::Arc;
use tauri::State;

/// 获取所有历史会话列表
#[tauri::command]
pub fn get_sessions(
    logger: State<'_, Arc<DataLogger>>,
) -> Result<Vec<SessionInfo>, String> {
    logger.get_sessions()
}

/// 获取指定会话的数据记录
#[tauri::command]
pub fn get_session_data(
    logger: State<'_, Arc<DataLogger>>,
    session_id: i64,
    direction: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<DataRecord>, String> {
    logger.get_session_data(
        session_id,
        direction,
        limit.unwrap_or(10000),
        offset.unwrap_or(0),
    )
}

/// 删除指定会话及其所有数据
#[tauri::command]
pub fn delete_session(
    logger: State<'_, Arc<DataLogger>>,
    session_id: i64,
) -> Result<(), String> {
    logger.delete_session(session_id)
}

/// 导出指定会话为 CSV 格式
#[tauri::command]
pub fn export_session_csv(
    logger: State<'_, Arc<DataLogger>>,
    session_id: i64,
) -> Result<String, String> {
    logger.export_session_csv(session_id)
}
