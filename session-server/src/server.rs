use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{net::{TcpListener, TcpStream}, sync::{Mutex, RwLock}};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message as WsMessage};
use futures_util::{StreamExt, SinkExt};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use remote_hid_shared::{Message, MessagePayload, MessageType, AuthMessage, SessionControlMessage, ClientInfo};

use crate::config::Config;

#[derive(Clone)]
pub struct SessionServer {
    config: Config,
    state: Arc<ServerState>,
}

#[derive(Default)]
struct ServerState {
    // Map of client_id -> sender channel to HID client
    hid_clients: RwLock<HashMap<String, ClientConnection>>,
    // Map of commander_id -> connection
    commanders: RwLock<HashMap<String, ClientConnection>>,
    // Map of session_id -> (commander_id, client_id)
    sessions: RwLock<HashMap<Uuid, (String, String)>>,
}

#[derive(Clone)]
struct ClientConnection {
    peer: SocketAddr,
    tx: Arc<Mutex<tokio_tungstenite::WebSocketStream<TcpStream>>>,
}

impl SessionServer {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        Ok(Self {
            config,
            state: Arc::new(ServerState::default()),
        })
    }

    pub async fn run(self: &Arc<Self>) -> anyhow::Result<()> {
        let addr = format!("{}:{}", self.config.server.host, self.config.server.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("Listening on {}", addr);

        loop {
            let (stream, peer) = listener.accept().await?;
            let server = Arc::clone(self);
            tokio::spawn(async move {
                if let Err(e) = server.handle_connection(stream, peer).await {
                    warn!("Connection {} error: {}", peer, e);
                }
            });
        }
    }

    async fn handle_connection(&self, stream: TcpStream, peer: SocketAddr) -> anyhow::Result<()> {
        let ws_stream = accept_async(stream).await?;
        info!("New WebSocket connection from {}", peer);
        let tx = Arc::new(Mutex::new(ws_stream));
        
        // Simple handshake: expect an auth request first
        // Note: For brevity, this example omits JWT validation; add per DESIGN.md
        // Read first message for identification
        let mut guard = tx.lock().await;
        let msg = match guard.next().await {
            Some(Ok(WsMessage::Text(text))) => text,
            Some(Ok(_)) => {
                return Ok(());
            }
            _ => return Ok(()),
        };
        drop(guard);

        let parsed: Message = serde_json::from_str(&msg)?;
        match (&parsed.message_type, &parsed.payload) {
            (MessageType::SessionControl, MessagePayload::SessionControl(SessionControlMessage::CreateSession { client_id, client_name })) => {
                self.register_hid_client(client_id.clone(), tx.clone(), peer, client_name.clone()).await;
                self.serve_hid_client(client_id.clone(), tx.clone(), peer).await
            }
            (MessageType::SessionControl, MessagePayload::SessionControl(SessionControlMessage::JoinSession { target_client_id })) => {
                self.register_commander(peer.to_string(), tx.clone(), peer).await;
                self.serve_commander(peer.to_string(), target_client_id.clone(), tx.clone(), peer).await
            }
            _ => {
                warn!("{} sent unexpected first message: {:?}", peer, parsed.message_type);
                Ok(())
            }
        }
    }

    async fn register_hid_client(&self, client_id: String, tx: Arc<Mutex<tokio_tungstenite::WebSocketStream<TcpStream>>>, peer: SocketAddr, client_name: Option<String>) {
        let mut map = self.state.hid_clients.write().await;
        map.insert(client_id.clone(), ClientConnection { peer, tx });
        info!("Registered HID client {} from {} ({:?})", client_id, peer, client_name);
    }

    async fn register_commander(&self, commander_id: String, tx: Arc<Mutex<tokio_tungstenite::WebSocketStream<TcpStream>>>, peer: SocketAddr) {
        let mut map = self.state.commanders.write().await;
        map.insert(commander_id.clone(), ClientConnection { peer, tx });
        info!("Registered Commander {} from {}", commander_id, peer);
    }

    async fn serve_hid_client(&self, client_id: String, tx: Arc<Mutex<tokio_tungstenite::WebSocketStream<TcpStream>>>, peer: SocketAddr) -> anyhow::Result<()> {
        let mut rx = tx.lock().await;
        while let Some(msg) = rx.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    if let Ok(message) = serde_json::from_str::<Message>(&text) {
                        debug!("HID client {} -> server: {:?}", client_id, message.message_type);
                        // For now we only handle status/heartbeat from HID client
                    }
                }
                Ok(WsMessage::Close(_)) => {
                    info!("HID client {} disconnected", client_id);
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    error!("HID client {} error: {}", client_id, e);
                    break;
                }
            }
        }
        // Cleanup
        self.state.hid_clients.write().await.remove(&client_id);
        Ok(())
    }

    async fn serve_commander(&self, commander_id: String, target_client_id: String, tx: Arc<Mutex<tokio_tungstenite::WebSocketStream<TcpStream>>>, peer: SocketAddr) -> anyhow::Result<()> {
        // Create a session id
        let session_id = Uuid::new_v4();
        self.state.sessions.write().await.insert(session_id, (commander_id.clone(), target_client_id.clone()));
        
        info!("Commander {} controlling HID client {} in session {}", commander_id, target_client_id, session_id);
        
        // Forward messages from commander to target HID client
        let mut commander_ws = tx.lock().await;
        while let Some(msg) = commander_ws.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    if let Ok(message) = serde_json::from_str::<Message>(&text) {
                        match message.message_type {
                            MessageType::HidEvent => {
                                // Forward to HID client
                                if let Some(conn) = self.state.hid_clients.read().await.get(&target_client_id).cloned() {
                                    let mut hid_ws = conn.tx.lock().await;
                                    if let Err(e) = hid_ws.send(WsMessage::Text(text)).await {
                                        error!("Failed to forward to HID client {}: {}", target_client_id, e);
                                    }
                                } else {
                                    warn!("HID client {} not connected", target_client_id);
                                }
                            }
                            MessageType::SessionControl => {
                                // Handle EndSession, etc.
                            }
                            _ => {}
                        }
                    }
                }
                Ok(WsMessage::Close(_)) => {
                    info!("Commander {} disconnected", commander_id);
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    error!("Commander {} error: {}", commander_id, e);
                    break;
                }
            }
        }

        // Cleanup session
        self.state.sessions.write().await.retain(|_, v| !(v.0 == commander_id && v.1 == target_client_id));
        Ok(())
    }
}
