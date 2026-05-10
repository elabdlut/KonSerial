//! 数据日志模块
//! 基于 SQLite 实现串口数据的持久化存储，支持会话管理、数据查询与导出

pub mod commands;

use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;

/// 获取默认数据库路径（与配置文件同目录）
/// Linux: ~/.config/konserial/data.db
/// macOS: ~/Library/Application Support/konserial/data.db
/// Windows: C:\Users\<User>\AppData\Roaming\konserial\data.db
pub fn default_db_path() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    config_dir.join("konserial").join("data.db")
}

// ========== 数据类型 ==========

/// 会话信息（返回给前端）
#[derive(Debug, Clone, Serialize)]
pub struct SessionInfo {
    pub id: i64,
    pub connection_id: String,
    pub session_type: String, // "serial", "tcp", "udp", "ws", "mqtt"
    pub port_name: String,
    pub baud_rate: u32,
    pub protocol: String,
    pub host: String,
    pub port: u16,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

/// 数据记录（返回给前端）
#[derive(Debug, Clone, Serialize)]
pub struct DataRecord {
    pub id: i64,
    pub session_id: i64,
    pub direction: String,
    pub data: Vec<u8>,
    pub timestamp: String,
}

// ========== DataLogger ==========

/// 数据日志管理器（线程安全，基于 SQLite）
pub struct DataLogger {
    conn: Mutex<Connection>,
}

fn row_to_data_record(row: &rusqlite::Row) -> rusqlite::Result<DataRecord> {
    Ok(DataRecord {
        id: row.get(0)?,
        session_id: row.get(1)?,
        direction: row.get(2)?,
        data: row.get(3)?,
        timestamp: row.get(4)?,
    })
}

impl DataLogger {
    /// 创建 DataLogger 实例，自动初始化数据库表结构
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, String> {
        let path = db_path.as_ref();

        // 确保数据库目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("无法创建数据库目录: {}", e))?;
        }

        let conn =
            Connection::open(path).map_err(|e| format!("无法打开数据库: {}", e))?;

        // 启用 WAL 模式 + 外键约束
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;",
        )
        .map_err(|e| format!("设置 PRAGMA 失败: {}", e))?;

        // 创建/迁移表结构
        Self::migrate_schema(&conn).await?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    async fn migrate_schema(conn: &Connection) -> Result<(), String> {
        // 尝试创建新 sessions 表（如果不存在）
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                connection_id TEXT    NOT NULL,
                session_type  TEXT    NOT NULL DEFAULT 'serial',
                port_name     TEXT    NOT NULL DEFAULT '',
                baud_rate     INTEGER NOT NULL DEFAULT 0,
                protocol      TEXT,
                host          TEXT,
                port          INTEGER,
                started_at    DATETIME DEFAULT (datetime('now','localtime')),
                ended_at      DATETIME
            );

            CREATE TABLE IF NOT EXISTS serial_data (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                direction  TEXT    NOT NULL CHECK(direction IN ('TX','RX')),
                data       BLOB   NOT NULL,
                timestamp  DATETIME DEFAULT (datetime('now','localtime'))
            );

            CREATE INDEX IF NOT EXISTS idx_serial_data_session
                ON serial_data(session_id, timestamp);",
        )
        .map_err(|e| format!("创建表失败: {}", e))?;

        // 检查旧 schema 是否需要迁移（没有 session_type 列）
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(sessions)")
            .map_err(|e| e.to_string())?
            .query_map([], |row| row.get::<_, String>("name"))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        if !cols.contains(&"session_type".to_string()) {
            conn.execute_batch(
                "ALTER TABLE sessions ADD COLUMN session_type TEXT NOT NULL DEFAULT 'serial';
                 ALTER TABLE sessions ADD COLUMN protocol TEXT;
                 ALTER TABLE sessions ADD COLUMN host TEXT;
                 ALTER TABLE sessions ADD COLUMN port INTEGER;",
            )
            .map_err(|e| format!("迁移 sessions 表失败: {}", e))?;
        }

        Ok(())
    }

    // ========== 会话管理 ==========

    /// 创建新的数据记录会话（打开串口时调用）
    pub async fn create_session(
        &self,
        connection_id: &str,
        port_name: &str,
        baud_rate: u32,
    ) -> Result<i64, String> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO sessions (connection_id, session_type, port_name, baud_rate) VALUES (?1, 'serial', ?2, ?3)",
            params![connection_id, port_name, baud_rate],
        )
        .map_err(|e| format!("创建会话失败: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    /// 创建网络连接的数据记录会话
    pub async fn create_network_session(
        &self,
        connection_id: &str,
        session_type: &str,
        protocol: &str,
        host: &str,
        port: u16,
    ) -> Result<i64, String> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO sessions (connection_id, session_type, protocol, host, port) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![connection_id, session_type, protocol, host, port],
        )
        .map_err(|e| format!("创建网络会话失败: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    /// 结束会话（关闭连接时调用）
    pub async fn end_session(&self, session_id: i64) -> Result<(), String> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE sessions SET ended_at = datetime('now','localtime') WHERE id = ?1",
            params![session_id],
        )
        .map_err(|e| format!("结束会话失败: {}", e))?;
        Ok(())
    }

    // ========== 数据写入 ==========

    /// 记录接收数据
    pub async fn log_rx(&self, session_id: i64, data: &[u8]) -> Result<(), String> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO serial_data (session_id, direction, data) VALUES (?1, 'RX', ?2)",
            params![session_id, data],
        )
        .map_err(|e| format!("记录 RX 数据失败: {}", e))?;
        Ok(())
    }

    /// 记录发送数据
    pub async fn log_tx(&self, session_id: i64, data: &[u8]) -> Result<(), String> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO serial_data (session_id, direction, data) VALUES (?1, 'TX', ?2)",
            params![session_id, data],
        )
        .map_err(|e| format!("记录 TX 数据失败: {}", e))?;
        Ok(())
    }

    // ========== 查询 ==========

    /// 获取所有会话列表（按时间倒序）
    pub async fn get_sessions(&self) -> Result<Vec<SessionInfo>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(
                "SELECT s.id, s.connection_id, s.session_type, s.port_name, s.baud_rate,
                        s.protocol, s.host, s.port,
                        s.started_at, s.ended_at,
                        COALESCE(SUM(CASE WHEN d.direction='RX' THEN length(d.data) ELSE 0 END),0),
                        COALESCE(SUM(CASE WHEN d.direction='TX' THEN length(d.data) ELSE 0 END),0)
                 FROM sessions s
                 LEFT JOIN serial_data d ON d.session_id = s.id
                 GROUP BY s.id
                 ORDER BY s.started_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(SessionInfo {
                    id: row.get(0)?,
                    connection_id: row.get(1)?,
                    session_type: row.get(2).unwrap_or_else(|_| "serial".to_string()),
                    port_name: row.get(3).unwrap_or_else(|_| "".to_string()),
                    baud_rate: row.get::<_, u32>(4).unwrap_or(0),
                    protocol: row.get(5).unwrap_or_else(|_| "".to_string()),
                    host: row.get(6).unwrap_or_else(|_| "".to_string()),
                    port: row.get::<_, u16>(7).unwrap_or(0),
                    started_at: row.get(8)?,
                    ended_at: row.get(9)?,
                    rx_bytes: row.get::<_, i64>(10)? as u64,
                    tx_bytes: row.get::<_, i64>(11)? as u64,
                })
            })
            .map_err(|e| e.to_string())?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// 获取指定会话的数据记录
    pub async fn get_session_data(
        &self,
        session_id: i64,
        direction: Option<String>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DataRecord>, String> {
        let conn = self.conn.lock().await;

        let records = if let Some(dir) = direction {
            let mut stmt = conn
                .prepare(
                    "SELECT id, session_id, direction, data, timestamp
                     FROM serial_data
                     WHERE session_id = ?1 AND direction = ?2
                     ORDER BY timestamp ASC LIMIT ?3 OFFSET ?4",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt.query_map(params![session_id, dir, limit, offset], row_to_data_record)
                .map_err(|e| e.to_string())?;
            let result: Vec<DataRecord> = rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            result
        } else {
            let mut stmt = conn
                .prepare(
                    "SELECT id, session_id, direction, data, timestamp
                     FROM serial_data
                     WHERE session_id = ?1
                     ORDER BY timestamp ASC LIMIT ?2 OFFSET ?3",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt.query_map(params![session_id, limit, offset], row_to_data_record)
                .map_err(|e| e.to_string())?;
            let result: Vec<DataRecord> = rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            result
        };

        Ok(records)
    }

    // ========== 删除与导出 ==========

    /// 删除指定会话及其所有数据
    pub async fn delete_session(&self, session_id: i64) -> Result<(), String> {
        let conn = self.conn.lock().await;
        // foreign_keys=ON + ON DELETE CASCADE 会自动清理 serial_data
        conn.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])
            .map_err(|e| format!("删除会话失败: {}", e))?;
        Ok(())
    }

    /// 导出指定会话为 CSV 格式字符串（分批读取避免 OOM）
    pub async fn export_session_csv(&self, session_id: i64) -> Result<String, String> {
        let mut csv = String::from("timestamp,direction,data_hex\n");
        let mut offset = 0u32;
        const BATCH_SIZE: u32 = 1000;

        loop {
            let batch = self.get_session_data(session_id, None, BATCH_SIZE, offset).await?;
            if batch.is_empty() {
                break;
            }
            for record in &batch {
                let hex: String = record
                    .data
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                csv.push_str(&format!("{},{},{}\n", record.timestamp, record.direction, hex));
            }
            offset += BATCH_SIZE;
        }
        Ok(csv)
    }
}
