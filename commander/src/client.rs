use anyhow::Result;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use futures_util::{StreamExt, SinkExt};
use tracing::{info, warn, error, debug};
use tokio::sync::mpsc;

use remote_hid_shared::{Message, MessagePayload, MessageType, SessionControlMessage, HidEvent};
use crate::input_capture::{InputCapture, InputEvent};

pub struct Commander {
    server_url: String,
    target_client_id: String,
}

impl Commander {
    pub fn new(server_url: String, target_client_id: String) -> Result<Self> {
        Ok(Self {
            server_url,
            target_client_id,
        })
    }
    
    pub async fn run(&self) -> Result<()> {
        info!("Connecting to session server at {}", self.server_url);
        
        let (ws_stream, _) = connect_async(&self.server_url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        // Send initial join session message
        let join_session = Message::session_control(
            None,
            SessionControlMessage::JoinSession {
                target_client_id: self.target_client_id.clone(),
            },
        );
        
        let msg_json = serde_json::to_string(&join_session)?;
        ws_sender.send(WsMessage::Text(msg_json)).await?;
        
        info!("Joined session for HID client: {}", self.target_client_id);
        
        // Start input capture
        let (input_tx, mut input_rx) = mpsc::unbounded_channel();
        let mut input_capture = InputCapture::new(input_tx)?;
        
        let _input_handle = tokio::spawn(async move {
            if let Err(e) = input_capture.start().await {
                error!("Input capture error: {}", e);
            }
        });
        
        // Main event loop
        loop {
            tokio::select! {
                // Handle input events from local capture
                Some(input_event) = input_rx.recv() => {
                    if let Some(hid_event) = self.convert_input_to_hid(input_event) {
                        let message = Message::hid_event(uuid::Uuid::new_v4(), hid_event);
                        let msg_json = serde_json::to_string(&message)?;
                        
                        if let Err(e) = ws_sender.send(WsMessage::Text(msg_json)).await {
                            error!("Failed to send HID event: {}", e);
                            break;
                        }
                    }
                }
                
                // Handle messages from server
                msg = ws_receiver.next() => {
                    match msg {
                        Some(Ok(WsMessage::Text(text))) => {
                            if let Ok(message) = serde_json::from_str::<Message>(&text) {
                                self.handle_server_message(message).await?;
                            }
                        }
                        Some(Ok(WsMessage::Close(_))) => {
                            info!("Server closed connection");
                            break;
                        }
                        Some(Ok(_)) => {}
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        None => break,
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_server_message(&self, message: Message) -> Result<()> {
        match message.message_type {
            MessageType::SessionControl => {
                if let MessagePayload::SessionControl(control) = message.payload {
                    match control {
                        SessionControlMessage::SessionEnded { reason } => {
                            info!("Session ended: {}", reason);
                        }
                        _ => {}
                    }
                }
            }
            MessageType::Status => {
                debug!("Received status message from server");
            }
            _ => {
                debug!("Ignoring server message type: {:?}", message.message_type);
            }
        }
        
        Ok(())
    }
    
    fn convert_input_to_hid(&self, input: InputEvent) -> Option<HidEvent> {
        match input {
            InputEvent::MouseMove { x, y, absolute } => {
                Some(HidEvent::MouseMove { x, y, absolute })
            }
            InputEvent::MouseClick { button, pressed, x, y } => {
                Some(HidEvent::MouseClick { button, pressed, x, y })
            }
            InputEvent::MouseScroll { delta_x, delta_y, x, y } => {
                Some(HidEvent::MouseScroll { delta_x, delta_y, x, y })
            }
            InputEvent::KeyEvent { key, pressed, modifiers } => {
                Some(HidEvent::KeyEvent { key, pressed, modifiers })
            }
        }
    }
}