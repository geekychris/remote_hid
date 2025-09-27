use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Main message wrapper for all communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message_type: MessageType,
    pub session_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub payload: MessagePayload,
}

/// Types of messages that can be sent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageType {
    /// Authentication related messages
    Auth,
    /// HID input events (keyboard, mouse)
    HidEvent,
    /// Session control and management
    SessionControl,
    /// System status and health
    Status,
}

/// Message payload containing the actual data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessagePayload {
    Auth(AuthMessage),
    HidEvent(HidEvent),
    SessionControl(SessionControlMessage),
    Status(StatusMessage),
}

/// Authentication message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum AuthMessage {
    /// Request authentication with credentials
    Request {
        username: String,
        password: String,
        client_type: ClientType,
        client_id: Option<String>,
    },
    /// Authentication response with token
    Response {
        success: bool,
        token: Option<String>,
        expires_at: Option<DateTime<Utc>>,
        error_message: Option<String>,
    },
    /// Refresh authentication token
    Refresh {
        refresh_token: String,
    },
    /// Logout and invalidate token
    Logout,
}

/// Client type identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientType {
    HidClient,
    Commander,
}

/// HID input event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum HidEvent {
    /// Mouse movement event
    MouseMove {
        x: i32,
        y: i32,
        /// Whether coordinates are absolute or relative
        absolute: bool,
    },
    /// Mouse button event
    MouseClick {
        button: MouseButton,
        pressed: bool,
        x: Option<i32>,
        y: Option<i32>,
    },
    /// Mouse scroll event
    MouseScroll {
        delta_x: i32,
        delta_y: i32,
        x: Option<i32>,
        y: Option<i32>,
    },
    /// Keyboard key event
    KeyEvent {
        key: KeyCode,
        pressed: bool,
        modifiers: KeyModifiers,
    },
}

/// Mouse button types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

/// Keyboard key codes (simplified set)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum KeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // Numbers
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    
    // Special keys
    Space, Enter, Tab, Backspace, Delete, Insert,
    Home, End, PageUp, PageDown,
    
    // Arrow keys
    ArrowUp, ArrowDown, ArrowLeft, ArrowRight,
    
    // Modifier keys
    LeftShift, RightShift, LeftControl, RightControl,
    LeftAlt, RightAlt, LeftSuper, RightSuper,
    
    // Other common keys
    Escape, CapsLock, NumLock, ScrollLock,
    PrintScreen, Pause, Menu,
    
    // Symbols (common ones)
    Minus, Equal, LeftBracket, RightBracket,
    Semicolon, Quote, Grave, Backslash,
    Comma, Period, Slash,
}

/// Keyboard modifier state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub super_key: bool,
}

/// Session control messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum SessionControlMessage {
    /// Create a new session (HID Client)
    CreateSession {
        client_id: String,
        client_name: Option<String>,
    },
    /// Join an existing session (Commander)
    JoinSession {
        target_client_id: String,
    },
    /// List available HID clients
    ListClients,
    /// Response with available clients
    ClientList {
        clients: Vec<ClientInfo>,
    },
    /// End the current session
    EndSession,
    /// Session ended notification
    SessionEnded {
        reason: String,
    },
}

/// Information about a connected HID client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub client_id: String,
    pub client_name: Option<String>,
    pub platform: String,
    pub connected_at: DateTime<Utc>,
    pub commander_connected: bool,
}

/// Status and health messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status_type")]
pub enum StatusMessage {
    /// Heartbeat/ping message
    Heartbeat,
    /// Connection status
    ConnectionStatus {
        connected: bool,
        latency_ms: Option<u64>,
    },
    /// Error notification
    Error {
        error_code: String,
        error_message: String,
    },
}

impl Message {
    /// Create a new message with current timestamp
    pub fn new(message_type: MessageType, session_id: Option<Uuid>, payload: MessagePayload) -> Self {
        Self {
            message_type,
            session_id,
            timestamp: Utc::now(),
            payload,
        }
    }
    
    /// Create an authentication request message
    pub fn auth_request(username: String, password: String, client_type: ClientType, client_id: Option<String>) -> Self {
        Self::new(
            MessageType::Auth,
            None,
            MessagePayload::Auth(AuthMessage::Request {
                username,
                password,
                client_type,
                client_id,
            }),
        )
    }
    
    /// Create a HID event message
    pub fn hid_event(session_id: Uuid, event: HidEvent) -> Self {
        Self::new(
            MessageType::HidEvent,
            Some(session_id),
            MessagePayload::HidEvent(event),
        )
    }
    
    /// Create a session control message
    pub fn session_control(session_id: Option<Uuid>, control: SessionControlMessage) -> Self {
        Self::new(
            MessageType::SessionControl,
            session_id,
            MessagePayload::SessionControl(control),
        )
    }
    
    /// Create a status message
    pub fn status(session_id: Option<Uuid>, status: StatusMessage) -> Self {
        Self::new(
            MessageType::Status,
            session_id,
            MessagePayload::Status(status),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_serialization() {
        let msg = Message::auth_request(
            "user".to_string(),
            "pass".to_string(),
            ClientType::Commander,
            None,
        );
        
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        
        match deserialized.payload {
            MessagePayload::Auth(AuthMessage::Request { username, .. }) => {
                assert_eq!(username, "user");
            }
            _ => panic!("Wrong message type"),
        }
    }
    
    #[test]
    fn test_hid_event_serialization() {
        let event = HidEvent::MouseMove { x: 100, y: 200, absolute: true };
        let msg = Message::hid_event(Uuid::new_v4(), event);
        
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        
        match deserialized.payload {
            MessagePayload::HidEvent(HidEvent::MouseMove { x, y, absolute }) => {
                assert_eq!(x, 100);
                assert_eq!(y, 200);
                assert!(absolute);
            }
            _ => panic!("Wrong event type"),
        }
    }
}