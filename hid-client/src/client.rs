use anyhow::Result;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use futures_util::{StreamExt, SinkExt};
use tracing::{info, warn, error, debug};

use remote_hid_shared::{Message, MessagePayload, MessageType, SessionControlMessage, HidEvent};
use crate::hid::HidHandler;

pub struct HidClient {
    server_url: String,
    client_id: String,
    client_name: Option<String>,
    hid_handler: HidHandler,
}

impl HidClient {
    pub fn new(server_url: String, client_id: String, client_name: Option<String>) -> Result<Self> {
        let hid_handler = HidHandler::new()?;
        
        Ok(Self {
            server_url,
            client_id,
            client_name,
            hid_handler,
        })
    }
    
    pub async fn run(&self) -> Result<()> {
        info!("Connecting to session server at {}", self.server_url);
        
        let (ws_stream, _) = connect_async(&self.server_url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        // Send initial session creation message
        let create_session = Message::session_control(
            None,
            SessionControlMessage::CreateSession {
                client_id: self.client_id.clone(),
                client_name: self.client_name.clone(),
            },
        );
        
        let msg_json = serde_json::to_string(&create_session)?;
        ws_sender.send(WsMessage::Text(msg_json)).await?;
        
        info!("Registered as HID client: {}", self.client_id);
        
        // Main message loop
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    if let Ok(message) = serde_json::from_str::<Message>(&text) {
                        if let Err(e) = self.handle_message(message).await {
                            error!("Failed to handle message: {}", e);
                        }
                    } else {
                        warn!("Failed to parse message: {}", text);
                    }
                }
                Ok(WsMessage::Close(_)) => {
                    info!("Server closed connection");
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_message(&self, message: Message) -> Result<()> {
        match message.message_type {
            MessageType::HidEvent => {
                if let MessagePayload::HidEvent(event) = message.payload {
                    debug!("Executing HID event: {:?}", event);
                    self.hid_handler.execute_event(event).await?;
                }
            }
            MessageType::SessionControl => {
                if let MessagePayload::SessionControl(control) = message.payload {
                    match control {
                        SessionControlMessage::EndSession => {
                            info!("Session ended by server");
                            // Could gracefully shutdown here
                        }
                        SessionControlMessage::SessionEnded { reason } => {
                            info!("Session ended: {}", reason);
                        }
                        _ => {}
                    }
                }
            }
            MessageType::Status => {
                // Handle status messages (heartbeat, etc.)
                debug!("Received status message");
            }
            _ => {
                debug!("Ignoring message type: {:?}", message.message_type);
            }
        }
        
        Ok(())
    }
}