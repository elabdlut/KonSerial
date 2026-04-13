//! 网络连接管理模块（TCP / UDP / WebSocket / MQTT / TCP Server / UDP Server）
use crate::data_logger::DataLogger;
use crate::log_info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Mutex, RwLock};
use futures_util::{SinkExt, StreamExt};

/// 网络协议类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NetProtocol {
    Tcp,
    Udp,
    #[serde(rename = "ws")]
    WebSocket,
    Mqtt,
    #[serde(rename = "tcp_server")]
    TcpServer,
    #[serde(rename = "udp_server")]
    UdpServer,
}

impl NetProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            NetProtocol::Tcp => "tcp",
            NetProtocol::Udp => "udp",
            NetProtocol::WebSocket => "ws",
            NetProtocol::Mqtt => "mqtt",
            NetProtocol::TcpServer => "tcp_server",
            NetProtocol::UdpServer => "udp_server",
        }
    }

    pub fn is_server(&self) -> bool {
        matches!(self, NetProtocol::TcpServer | NetProtocol::UdpServer)
    }
}

/// 网络连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetConnectionConfig {
    pub protocol: NetProtocol,
    pub host: String,
    pub port: u16,
    pub path: Option<String>,
    pub topic: Option<String>,
}

/// 网络连接状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetStatus {
    Disconnected,
    Connecting,
    Connected,
    Listening,
    Error(String),
}

/// 网络连接运行时信息
#[derive(Debug, Clone, Serialize)]
pub struct NetConnectionInfo {
    pub connection_id: String,
    pub status: NetStatus,
    pub config: NetConnectionConfig,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub last_error: Option<String>,
    pub created_at: String,
}

/// 网络数据事件载荷（推送给前端）
#[derive(Clone, Serialize)]
pub struct NetworkDataPayload {
    pub connection_id: String,
    pub data: Vec<u8>,
    pub peer_id: Option<String>,
}

/// 网络Peer事件（Server模式下客户端连接/断开）
#[derive(Clone, Serialize)]
pub struct NetworkPeerEvent {
    pub connection_id: String,
    pub peer_id: String,
    pub event: String, // "connected" | "disconnected"
}

/// 全局运行时信息
#[derive(Debug, Clone, Serialize)]
pub struct NetworkGlobalInfo {
    pub active_connections: Vec<NetConnectionInfo>,
    pub total_connections: usize,
}

/// 内部写端抽象
enum NetWriter {
    Tcp(Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>),
    Udp(Arc<tokio::net::UdpSocket>),
    Ws(tokio::sync::mpsc::UnboundedSender<tokio_tungstenite::tungstenite::protocol::Message>),
    Mqtt(rumqttc::AsyncClient),
    TcpServer {
        peers: Arc<Mutex<HashMap<String, Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>>>>,
    },
    UdpServer {
        socket: Arc<tokio::net::UdpSocket>,
        peers: Arc<Mutex<HashMap<String, std::net::SocketAddr>>>,
    },
}

/// 单个网络连接（内部结构）
struct NetConnection {
    writer: NetWriter,
    info: Arc<Mutex<NetConnectionInfo>>,
    read_task: tokio::task::JoinHandle<()>,
    accept_task: Option<tokio::task::JoinHandle<()>>,
    running: Arc<AtomicBool>,
    bytes_received_counter: Arc<AtomicU64>,
    session_id: i64,
}

/// 网络连接管理器
pub struct NetworkManager {
    connections: Arc<RwLock<HashMap<String, NetConnection>>>,
    data_logger: Arc<DataLogger>,
}

impl NetworkManager {
    pub fn new(data_logger: Arc<DataLogger>) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            data_logger,
        }
    }

    /// 打开新的网络连接
    pub async fn open(
        &self,
        connection_id: String,
        config: NetConnectionConfig,
        app_handle: AppHandle,
    ) -> Result<(), String> {
        log_info!(&format!(
            "[{}] 正在打开网络连接: {:?} {}:{}",
            connection_id, config.protocol, config.host, config.port
        ));

        {
            let conns = self.connections.read().await;
            if conns.contains_key(&connection_id) {
                return Err(format!("连接 {} 已存在", connection_id));
            }
        }

        let running = Arc::new(AtomicBool::new(true));
        let bytes_counter = Arc::new(AtomicU64::new(0));

        let protocol_str = config.protocol.as_str();
        let session_id = self.data_logger
            .create_network_session(&connection_id, protocol_str, protocol_str, &config.host, config.port)
            .map_err(|e| format!("创建网络会话失败: {}", e))?;

        let connection = match config.protocol {
            NetProtocol::Tcp => {
                self.build_tcp(connection_id.clone(), config, app_handle, running.clone(), bytes_counter.clone(), self.data_logger.clone(), session_id)
                    .await?
            }
            NetProtocol::Udp => {
                self.build_udp(connection_id.clone(), config, app_handle, running.clone(), bytes_counter.clone(), self.data_logger.clone(), session_id)
                    .await?
            }
            NetProtocol::WebSocket => {
                self.build_ws(connection_id.clone(), config, app_handle, running.clone(), bytes_counter.clone(), self.data_logger.clone(), session_id)
                    .await?
            }
            NetProtocol::Mqtt => {
                self.build_mqtt(connection_id.clone(), config, app_handle, running.clone(), bytes_counter.clone(), self.data_logger.clone(), session_id)
                    .await?
            }
            NetProtocol::TcpServer => {
                self.build_tcp_server(connection_id.clone(), config, app_handle, running.clone(), bytes_counter.clone(), self.data_logger.clone(), session_id)
                    .await?
            }
            NetProtocol::UdpServer => {
                self.build_udp_server(connection_id.clone(), config, app_handle, running.clone(), bytes_counter.clone(), self.data_logger.clone(), session_id)
                    .await?
            }
        };

        self.connections.write().await.insert(connection_id.clone(), connection);
        log_info!(&format!("[{}] 网络连接打开成功", connection_id));
        Ok(())
    }

    async fn build_tcp(
        &self,
        connection_id: String,
        config: NetConnectionConfig,
        app_handle: AppHandle,
        running: Arc<AtomicBool>,
        bytes_counter: Arc<AtomicU64>,
        data_logger: Arc<DataLogger>,
        session_id: i64,
    ) -> Result<NetConnection, String> {
        let stream = tokio::net::TcpStream::connect((config.host.as_str(), config.port))
            .await
            .map_err(|e| format!("TCP 连接失败: {}", e))?;

        let (mut read_half, write_half) = stream.into_split();
        let writer = NetWriter::Tcp(Arc::new(Mutex::new(write_half)));

        let info = Arc::new(Mutex::new(NetConnectionInfo {
            connection_id: connection_id.clone(),
            status: NetStatus::Connected,
            config,
            bytes_received: 0,
            bytes_sent: 0,
            last_error: None,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }));
        let info_clone = info.clone();

        let conn_id = connection_id.clone();
        let running_clone = running.clone();
        let counter_clone = bytes_counter.clone();
        let read_task = tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            while running_clone.load(Ordering::Relaxed) {
                match read_half.read(&mut buf).await {
                    Ok(0) | Err(_) => {
                        let mut info = info_clone.lock().await;
                        info.status = NetStatus::Error("连接断开".to_string());
                        info.last_error = Some("连接断开".to_string());
                        break;
                    }
                    Ok(n) => {
                        counter_clone.fetch_add(n as u64, Ordering::Relaxed);
                        let data = buf[..n].to_vec();
                        let _ = data_logger.log_rx(session_id, &data);
                        let _ = app_handle.emit("network-data", NetworkDataPayload {
                            connection_id: conn_id.clone(),
                            data,
                            peer_id: None,
                        });
                    }
                }
            }
        });

        Ok(NetConnection {
            writer,
            info,
            read_task,
            accept_task: None,
            running,
            bytes_received_counter: bytes_counter,
            session_id,
        })
    }

    async fn build_udp(
        &self,
        connection_id: String,
        config: NetConnectionConfig,
        app_handle: AppHandle,
        running: Arc<AtomicBool>,
        bytes_counter: Arc<AtomicU64>,
        data_logger: Arc<DataLogger>,
        session_id: i64,
    ) -> Result<NetConnection, String> {
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| format!("UDP 绑定失败: {}", e))?;
        socket
            .connect((config.host.as_str(), config.port))
            .await
            .map_err(|e| format!("UDP 连接失败: {}", e))?;

        let socket_arc = Arc::new(socket);
        let socket_read = socket_arc.clone();
        let writer = NetWriter::Udp(socket_arc);

        let info = Arc::new(Mutex::new(NetConnectionInfo {
            connection_id: connection_id.clone(),
            status: NetStatus::Connected,
            config,
            bytes_received: 0,
            bytes_sent: 0,
            last_error: None,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }));
        let info_clone = info.clone();

        let conn_id = connection_id.clone();
        let running_clone = running.clone();
        let counter_clone = bytes_counter.clone();
        let read_task = tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            while running_clone.load(Ordering::Relaxed) {
                match socket_read.recv(&mut buf).await {
                    Err(_) => {
                        let mut info = info_clone.lock().await;
                        info.status = NetStatus::Error("连接断开".to_string());
                        info.last_error = Some("连接断开".to_string());
                        break;
                    }
                    Ok(n) => {
                        counter_clone.fetch_add(n as u64, Ordering::Relaxed);
                        let data = buf[..n].to_vec();
                        let _ = data_logger.log_rx(session_id, &data);
                        let _ = app_handle.emit("network-data", NetworkDataPayload {
                            connection_id: conn_id.clone(),
                            data,
                            peer_id: None,
                        });
                    }
                }
            }
        });

        Ok(NetConnection {
            writer,
            info,
            read_task,
            accept_task: None,
            running,
            bytes_received_counter: bytes_counter,
            session_id,
        })
    }

    async fn build_ws(
        &self,
        connection_id: String,
        config: NetConnectionConfig,
        app_handle: AppHandle,
        running: Arc<AtomicBool>,
        bytes_counter: Arc<AtomicU64>,
        data_logger: Arc<DataLogger>,
        session_id: i64,
    ) -> Result<NetConnection, String> {
        let path = config.path.clone().unwrap_or_default();
        let url = format!("ws://{}:{}{}", config.host, config.port, path);
        let (ws_stream, _) = tokio_tungstenite::connect_async(&url)
            .await
            .map_err(|e| format!("WebSocket 连接失败: {}", e))?;

        let (write, mut read) = ws_stream.split();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let writer = NetWriter::Ws(tx);

        tokio::spawn(async move {
            let mut write = write;
            while let Some(msg) = rx.recv().await {
                if write.send(msg).await.is_err() {
                    break;
                }
            }
        });

        let info = Arc::new(Mutex::new(NetConnectionInfo {
            connection_id: connection_id.clone(),
            status: NetStatus::Connected,
            config,
            bytes_received: 0,
            bytes_sent: 0,
            last_error: None,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }));
        let info_clone = info.clone();

        let conn_id = connection_id.clone();
        let running_clone = running.clone();
        let counter_clone = bytes_counter.clone();
        let read_task = tokio::spawn(async move {
            use tokio_tungstenite::tungstenite::protocol::Message;
            while running_clone.load(Ordering::Relaxed) {
                match read.next().await {
                    Some(Ok(Message::Binary(data))) => {
                        let vec_data = data.to_vec();
                        counter_clone.fetch_add(vec_data.len() as u64, Ordering::Relaxed);
                        let _ = data_logger.log_rx(session_id, &vec_data);
                        let _ = app_handle.emit("network-data", NetworkDataPayload {
                            connection_id: conn_id.clone(),
                            data: vec_data,
                            peer_id: None,
                        });
                    }
                    Some(Ok(Message::Text(text))) => {
                        let vec_data = text.to_string().into_bytes();
                        counter_clone.fetch_add(vec_data.len() as u64, Ordering::Relaxed);
                        let _ = data_logger.log_rx(session_id, &vec_data);
                        let _ = app_handle.emit("network-data", NetworkDataPayload {
                            connection_id: conn_id.clone(),
                            data: vec_data,
                            peer_id: None,
                        });
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        let mut info = info_clone.lock().await;
                        info.status = NetStatus::Error("连接断开".to_string());
                        info.last_error = Some("连接断开".to_string());
                        break;
                    }
                    Some(Err(_)) => {
                        let mut info = info_clone.lock().await;
                        info.status = NetStatus::Error("连接断开".to_string());
                        info.last_error = Some("连接断开".to_string());
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(NetConnection {
            writer,
            info,
            read_task,
            accept_task: None,
            running,
            bytes_received_counter: bytes_counter,
            session_id,
        })
    }

    async fn build_mqtt(
        &self,
        connection_id: String,
        config: NetConnectionConfig,
        app_handle: AppHandle,
        running: Arc<AtomicBool>,
        bytes_counter: Arc<AtomicU64>,
        data_logger: Arc<DataLogger>,
        session_id: i64,
    ) -> Result<NetConnection, String> {
        use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};

        let client_id = config
            .path
            .clone()
            .unwrap_or_else(|| format!("konserial-{}", connection_id));
        let mut mqttoptions = MqttOptions::new(client_id, config.host.clone(), config.port);
        mqttoptions.set_keep_alive(std::time::Duration::from_secs(5));

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        let writer = NetWriter::Mqtt(client.clone());

        if let Some(ref topic) = config.topic {
            client
                .subscribe(topic.clone(), QoS::AtMostOnce)
                .await
                .map_err(|e| format!("MQTT 订阅失败: {}", e))?;
        }

        let info = Arc::new(Mutex::new(NetConnectionInfo {
            connection_id: connection_id.clone(),
            status: NetStatus::Connected,
            config,
            bytes_received: 0,
            bytes_sent: 0,
            last_error: None,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }));
        let info_clone = info.clone();

        let conn_id = connection_id.clone();
        let running_clone = running.clone();
        let counter_clone = bytes_counter.clone();
        let read_task = tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Packet::Publish(p))) => {
                        let data = p.payload.to_vec();
                        counter_clone.fetch_add(data.len() as u64, Ordering::Relaxed);
                        let _ = data_logger.log_rx(session_id, &data);
                        let _ = app_handle.emit("network-data", NetworkDataPayload {
                            connection_id: conn_id.clone(),
                            data,
                            peer_id: None,
                        });
                    }
                    Ok(Event::Incoming(Packet::Disconnect)) => {
                        let mut info = info_clone.lock().await;
                        info.status = NetStatus::Error("连接断开".to_string());
                        info.last_error = Some("连接断开".to_string());
                        break;
                    }
                    Err(_) => {
                        let mut info = info_clone.lock().await;
                        info.status = NetStatus::Error("连接断开".to_string());
                        info.last_error = Some("连接断开".to_string());
                        break;
                    }
                    _ => {}
                }
                if !running_clone.load(Ordering::Relaxed) {
                    break;
                }
            }
        });

        Ok(NetConnection {
            writer,
            info,
            read_task,
            accept_task: None,
            running,
            bytes_received_counter: bytes_counter,
            session_id,
        })
    }

    async fn build_tcp_server(
        &self,
        connection_id: String,
        config: NetConnectionConfig,
        app_handle: AppHandle,
        running: Arc<AtomicBool>,
        bytes_counter: Arc<AtomicU64>,
        data_logger: Arc<DataLogger>,
        session_id: i64,
    ) -> Result<NetConnection, String> {
        let listener = tokio::net::TcpListener::bind((config.host.as_str(), config.port))
            .await
            .map_err(|e| format!("TCP Server 绑定失败: {}", e))?;

        let peers: Arc<Mutex<HashMap<String, Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let info = Arc::new(Mutex::new(NetConnectionInfo {
            connection_id: connection_id.clone(),
            status: NetStatus::Listening,
            config,
            bytes_received: 0,
            bytes_sent: 0,
            last_error: None,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }));
        let info_clone = info.clone();

        let conn_id = connection_id.clone();
        let running_clone = running.clone();
        let peers_clone = peers.clone();
        let app_handle_clone = app_handle.clone();

        let bytes_counter_clone = bytes_counter.clone();
        let data_logger_clone = data_logger.clone();

        let accept_task = tokio::spawn(async move {
            while running_clone.load(Ordering::Relaxed) {
                match tokio::time::timeout(std::time::Duration::from_millis(500), listener.accept()).await {
                    Ok(Ok((stream, addr))) => {
                        let peer_id = addr.to_string();
                        let (mut read_half, write_half) = stream.into_split();
                        let write_arc = Arc::new(Mutex::new(write_half));
                        peers_clone.lock().await.insert(peer_id.clone(), write_arc.clone());

                        let _ = app_handle_clone.emit("network-peer-update", NetworkPeerEvent {
                            connection_id: conn_id.clone(),
                            peer_id: peer_id.clone(),
                            event: "connected".into(),
                        });

                        let conn_id_inner = conn_id.clone();
                        let running_inner = running_clone.clone();
                        let counter_inner = bytes_counter_clone.clone();
                        let info_inner = info_clone.clone();
                        let peers_inner = peers_clone.clone();
                        let app_handle_inner = app_handle_clone.clone();
                        let peer_id_inner = peer_id.clone();
                        let data_logger_inner = data_logger_clone.clone();

                        tokio::spawn(async move {
                            let mut buf = [0u8; 1024];
                            loop {
                                if !running_inner.load(Ordering::Relaxed) { break; }
                                match read_half.read(&mut buf).await {
                                    Ok(0) | Err(_) => break,
                                    Ok(n) => {
                                        counter_inner.fetch_add(n as u64, Ordering::Relaxed);
                                        let data = buf[..n].to_vec();
                                        let _ = data_logger_inner.log_rx(session_id, &data);
                                        let _ = app_handle_inner.emit("network-data", NetworkDataPayload {
                                            connection_id: conn_id_inner.clone(),
                                            data,
                                            peer_id: Some(peer_id_inner.clone()),
                                        });
                                    }
                                }
                            }
                            peers_inner.lock().await.remove(&peer_id_inner);
                            let _ = app_handle_inner.emit("network-peer-update", NetworkPeerEvent {
                                connection_id: conn_id_inner.clone(),
                                peer_id: peer_id_inner,
                                event: "disconnected".into(),
                            });
                            let remaining = peers_inner.lock().await.len();
                            if remaining == 0 {
                                let mut i = info_inner.lock().await;
                                if matches!(i.status, NetStatus::Connected) {
                                    i.status = NetStatus::Listening;
                                }
                            }
                        });

                        {
                            let mut i = info_clone.lock().await;
                            i.status = NetStatus::Connected;
                        }
                    }
                    Ok(Err(_)) => break,
                    Err(_) => continue,
                }
            }
        });

        Ok(NetConnection {
            writer: NetWriter::TcpServer { peers },
            info,
            read_task: tokio::spawn(async {}),
            accept_task: Some(accept_task),
            running,
            bytes_received_counter: bytes_counter,
            session_id,
        })
    }

    async fn build_udp_server(
        &self,
        connection_id: String,
        config: NetConnectionConfig,
        app_handle: AppHandle,
        running: Arc<AtomicBool>,
        bytes_counter: Arc<AtomicU64>,
        data_logger: Arc<DataLogger>,
        session_id: i64,
    ) -> Result<NetConnection, String> {
        let socket = tokio::net::UdpSocket::bind((config.host.as_str(), config.port))
            .await
            .map_err(|e| format!("UDP Server 绑定失败: {}", e))?;
        let socket_arc = Arc::new(socket);
        let socket_read = socket_arc.clone();

        let peers = Arc::new(Mutex::new(HashMap::<String, std::net::SocketAddr>::new()));
        let peers_clone = peers.clone();

        let info = Arc::new(Mutex::new(NetConnectionInfo {
            connection_id: connection_id.clone(),
            status: NetStatus::Listening,
            config,
            bytes_received: 0,
            bytes_sent: 0,
            last_error: None,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }));
        let info_clone = info.clone();

        let conn_id = connection_id.clone();
        let running_clone = running.clone();
        let counter_clone = bytes_counter.clone();
        let app_handle_clone = app_handle.clone();

        let read_task = tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            while running_clone.load(Ordering::Relaxed) {
                match socket_read.recv_from(&mut buf).await {
                    Err(_) => {
                        let mut info = info_clone.lock().await;
                        info.status = NetStatus::Error("接收失败".into());
                        info.last_error = Some("接收失败".into());
                        break;
                    }
                    Ok((n, addr)) => {
                        counter_clone.fetch_add(n as u64, Ordering::Relaxed);
                        let peer_id = addr.to_string();
                        if !peers_clone.lock().await.contains_key(&peer_id) {
                            peers_clone.lock().await.insert(peer_id.clone(), addr);
                            let _ = app_handle_clone.emit("network-peer-update", NetworkPeerEvent {
                                connection_id: conn_id.clone(),
                                peer_id: peer_id.clone(),
                                event: "connected".into(),
                            });
                            let mut info = info_clone.lock().await;
                            info.status = NetStatus::Connected;
                        }
                        let data = buf[..n].to_vec();
                        let _ = data_logger.log_rx(session_id, &data);
                        let _ = app_handle_clone.emit("network-data", NetworkDataPayload {
                            connection_id: conn_id.clone(),
                            data,
                            peer_id: Some(peer_id),
                        });
                    }
                }
            }
        });

        Ok(NetConnection {
            writer: NetWriter::UdpServer { socket: socket_arc, peers },
            info,
            read_task,
            accept_task: None,
            running,
            bytes_received_counter: bytes_counter,
            session_id,
        })
    }

    /// 发送数据到指定网络连接
    pub async fn send(
        &self,
        connection_id: &str,
        data: Vec<u8>,
        peer_id: Option<&str>,
    ) -> Result<usize, String> {
        let mut conns = self.connections.write().await;

        if let Some(conn) = conns.get_mut(connection_id) {
            let bytes = data.len();
            let data_for_log = data.clone();
            match &conn.writer {
                NetWriter::Tcp(write_half) => {
                    let mut w = write_half.lock().await;
                    w.write_all(&data)
                        .await
                        .map_err(|e| format!("TCP 发送失败: {}", e))?;
                }
                NetWriter::Udp(socket) => {
                    socket
                        .send(&data)
                        .await
                        .map_err(|e| format!("UDP 发送失败: {}", e))?;
                }
                NetWriter::Ws(tx) => {
                    use tokio_tungstenite::tungstenite::protocol::Message;
                    tx.send(Message::Binary(data.into()))
                        .map_err(|e| format!("WebSocket 发送失败: {}", e))?;
                }
                NetWriter::Mqtt(client) => {
                    use rumqttc::QoS;
                    let topic = conn
                        .info
                        .lock()
                        .await
                        .config
                        .topic
                        .clone()
                        .unwrap_or_else(|| "debug/pub".to_string());
                    client
                        .publish(&topic, QoS::AtMostOnce, false, data)
                        .await
                        .map_err(|e| format!("MQTT 发送失败: {}", e))?;
                }
                NetWriter::TcpServer { peers } => {
                    let peer_id = peer_id.ok_or("请选择要发送的 TCP 客户端")?;
                    let peers_guard = peers.lock().await;
                    let writer_arc = peers_guard
                        .get(peer_id)
                        .ok_or_else(|| format!("客户端 {} 不存在", peer_id))?
                        .clone();
                    drop(peers_guard);
                    let mut w = writer_arc.lock().await;
                    w.write_all(&data)
                        .await
                        .map_err(|e| format!("TCP Server 发送失败: {}", e))?;
                }
                NetWriter::UdpServer { socket, peers } => {
                    let peer_id = peer_id.ok_or("请选择要发送的 UDP 目标")?;
                    let peers_guard = peers.lock().await;
                    let addr = peers_guard
                        .get(peer_id)
                        .ok_or_else(|| format!("UDP 目标 {} 不存在", peer_id))?
                        .clone();
                    drop(peers_guard);
                    socket
                        .send_to(&data, addr)
                        .await
                        .map_err(|e| format!("UDP Server 发送失败: {}", e))?;
                }
            }
            conn.info.lock().await.bytes_sent += bytes as u64;
            let _ = self.data_logger.log_tx(conn.session_id, &data_for_log);
            Ok(bytes)
        } else {
            Err(format!("连接 {} 不存在", connection_id))
        }
    }

    /// 关闭指定连接
    pub async fn close(&self, connection_id: &str) -> Result<(), String> {
        let removed = self.connections.write().await.remove(connection_id);

        if let Some(conn) = removed {
            conn.running.store(false, Ordering::Relaxed);
            conn.read_task.abort();
            if let Some(t) = conn.accept_task {
                t.abort();
            }
            let _ = self.data_logger.end_session(conn.session_id);
            log_info!(&format!("[{}] 网络连接已关闭", connection_id));
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
            if let Some(t) = conn.accept_task {
                t.abort();
            }
            let _ = self.data_logger.end_session(conn.session_id);
        }
        log_info!(&format!("已关闭所有网络连接 ({}个)", count));
    }

    /// 获取指定连接的状态
    pub async fn get_connection_info(
        &self,
        connection_id: &str,
    ) -> Result<NetConnectionInfo, String> {
        let conns = self.connections.read().await;
        if let Some(c) = conns.get(connection_id) {
            let mut info = c.info.lock().await.clone();
            info.bytes_received = c.bytes_received_counter.load(Ordering::Relaxed);
            Ok(info)
        } else {
            Err(format!("连接 {} 不存在", connection_id))
        }
    }

    /// 获取所有连接的状态
    pub async fn get_all_connections(&self) -> Vec<NetConnectionInfo> {
        let conns = self.connections.read().await;
        let mut result = Vec::new();
        for c in conns.values() {
            let mut info = c.info.lock().await.clone();
            info.bytes_received = c.bytes_received_counter.load(Ordering::Relaxed);
            result.push(info);
        }
        result
    }

    /// 获取全局运行时信息
    pub async fn get_global_info(&self) -> NetworkGlobalInfo {
        let active_connections = self.get_all_connections().await;
        NetworkGlobalInfo {
            total_connections: active_connections.len(),
            active_connections,
        }
    }
}
