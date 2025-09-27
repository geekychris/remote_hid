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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

/// Keyboard key codes (simplified set)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
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
    
    #[test]
    fn test_auth_message_types() {
        // Test auth request
        let request = AuthMessage::Request {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            client_type: ClientType::HidClient,
            client_id: Some("client123".to_string()),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: AuthMessage = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            AuthMessage::Request { username, client_type, client_id, .. } => {
                assert_eq!(username, "testuser");
                assert!(matches!(client_type, ClientType::HidClient));
                assert_eq!(client_id, Some("client123".to_string()));
            }
            _ => panic!("Wrong auth message type"),
        }
        
        // Test auth response
        let response = AuthMessage::Response {
            success: true,
            token: Some("jwt_token_here".to_string()),
            expires_at: Some(Utc::now()),
            error_message: None,
        };
        
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: AuthMessage = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            AuthMessage::Response { success, token, .. } => {
                assert!(success);
                assert_eq!(token, Some("jwt_token_here".to_string()));
            }
            _ => panic!("Wrong auth message type"),
        }
    }
    
    #[test]
    fn test_hid_event_types() {
        // Test mouse events
        let mouse_move = HidEvent::MouseMove { x: 500, y: 300, absolute: false };
        let json = serde_json::to_string(&mouse_move).unwrap();
        let deserialized: HidEvent = serde_json::from_str(&json).unwrap();
        match deserialized {
            HidEvent::MouseMove { x, y, absolute } => {
                assert_eq!(x, 500);
                assert_eq!(y, 300);
                assert!(!absolute);
            }
            _ => panic!("Wrong event type"),
        }
        
        let mouse_click = HidEvent::MouseClick {
            button: MouseButton::Right,
            pressed: true,
            x: Some(100),
            y: Some(200),
        };
        let json = serde_json::to_string(&mouse_click).unwrap();
        let deserialized: HidEvent = serde_json::from_str(&json).unwrap();
        match deserialized {
            HidEvent::MouseClick { button, pressed, x, y } => {
                assert!(matches!(button, MouseButton::Right));
                assert!(pressed);
                assert_eq!(x, Some(100));
                assert_eq!(y, Some(200));
            }
            _ => panic!("Wrong event type"),
        }
        
        let mouse_scroll = HidEvent::MouseScroll {
            delta_x: -5,
            delta_y: 10,
            x: None,
            y: None,
        };
        let json = serde_json::to_string(&mouse_scroll).unwrap();
        let deserialized: HidEvent = serde_json::from_str(&json).unwrap();
        match deserialized {
            HidEvent::MouseScroll { delta_x, delta_y, x, y } => {
                assert_eq!(delta_x, -5);
                assert_eq!(delta_y, 10);
                assert_eq!(x, None);
                assert_eq!(y, None);
            }
            _ => panic!("Wrong event type"),
        }
        
        // Test keyboard events
        let key_event = HidEvent::KeyEvent {
            key: KeyCode::Space,
            pressed: false,
            modifiers: KeyModifiers {
                shift: true,
                control: false,
                alt: true,
                super_key: false,
            },
        };
        let json = serde_json::to_string(&key_event).unwrap();
        let deserialized: HidEvent = serde_json::from_str(&json).unwrap();
        match deserialized {
            HidEvent::KeyEvent { key, pressed, modifiers } => {
                assert!(matches!(key, KeyCode::Space));
                assert!(!pressed);
                assert!(modifiers.shift);
                assert!(!modifiers.control);
                assert!(modifiers.alt);
                assert!(!modifiers.super_key);
            }
            _ => panic!("Wrong event type"),
        }
    }
    
    #[test]
    fn test_session_control_messages() {
        // Test CreateSession
        let create = SessionControlMessage::CreateSession {
            client_id: "test_client".to_string(),
            client_name: Some("Test Client".to_string()),
        };
        let json = serde_json::to_string(&create).unwrap();
        let deserialized: SessionControlMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            SessionControlMessage::CreateSession { client_id, client_name } => {
                assert_eq!(client_id, "test_client");
                assert_eq!(client_name, Some("Test Client".to_string()));
            }
            _ => panic!("Wrong session control message type"),
        }
        
        // Test JoinSession
        let join = SessionControlMessage::JoinSession {
            target_client_id: "target123".to_string(),
        };
        let json = serde_json::to_string(&join).unwrap();
        let deserialized: SessionControlMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            SessionControlMessage::JoinSession { target_client_id } => {
                assert_eq!(target_client_id, "target123");
            }
            _ => panic!("Wrong session control message type"),
        }
        
        // Test ClientList
        let clients = vec![
            ClientInfo {
                client_id: "client1".to_string(),
                client_name: Some("Client 1".to_string()),
                platform: "macOS".to_string(),
                connected_at: Utc::now(),
                commander_connected: false,
            },
            ClientInfo {
                client_id: "client2".to_string(),
                client_name: None,
                platform: "Windows".to_string(),
                connected_at: Utc::now(),
                commander_connected: true,
            },
        ];
        
        let client_list = SessionControlMessage::ClientList {
            clients: clients.clone(),
        };
        let json = serde_json::to_string(&client_list).unwrap();
        let deserialized: SessionControlMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            SessionControlMessage::ClientList { clients: deserialized_clients } => {
                assert_eq!(deserialized_clients.len(), 2);
                assert_eq!(deserialized_clients[0].client_id, "client1");
                assert_eq!(deserialized_clients[1].platform, "Windows");
                assert!(deserialized_clients[1].commander_connected);
            }
            _ => panic!("Wrong session control message type"),
        }
    }
    
    #[test]
    fn test_status_messages() {
        // Test Heartbeat
        let heartbeat = StatusMessage::Heartbeat;
        let json = serde_json::to_string(&heartbeat).unwrap();
        let deserialized: StatusMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, StatusMessage::Heartbeat));
        
        // Test ConnectionStatus
        let status = StatusMessage::ConnectionStatus {
            connected: true,
            latency_ms: Some(42),
        };
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: StatusMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            StatusMessage::ConnectionStatus { connected, latency_ms } => {
                assert!(connected);
                assert_eq!(latency_ms, Some(42));
            }
            _ => panic!("Wrong status message type"),
        }
        
        // Test Error
        let error = StatusMessage::Error {
            error_code: "AUTH_FAILED".to_string(),
            error_message: "Invalid credentials".to_string(),
        };
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: StatusMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            StatusMessage::Error { error_code, error_message } => {
                assert_eq!(error_code, "AUTH_FAILED");
                assert_eq!(error_message, "Invalid credentials");
            }
            _ => panic!("Wrong status message type"),
        }
    }
    
    #[test]
    fn test_message_constructors() {
        let session_id = Uuid::new_v4();
        
        // Test auth_request constructor
        let auth_msg = Message::auth_request(
            "user".to_string(),
            "pass".to_string(),
            ClientType::Commander,
            Some("client_id".to_string()),
        );
        assert!(matches!(auth_msg.message_type, MessageType::Auth));
        assert!(auth_msg.session_id.is_none());
        
        // Test hid_event constructor
        let hid_msg = Message::hid_event(
            session_id,
            HidEvent::MouseMove { x: 10, y: 20, absolute: true },
        );
        assert!(matches!(hid_msg.message_type, MessageType::HidEvent));
        assert_eq!(hid_msg.session_id, Some(session_id));
        
        // Test session_control constructor
        let session_msg = Message::session_control(
            Some(session_id),
            SessionControlMessage::EndSession,
        );
        assert!(matches!(session_msg.message_type, MessageType::SessionControl));
        assert_eq!(session_msg.session_id, Some(session_id));
        
        // Test status constructor
        let status_msg = Message::status(
            None,
            StatusMessage::Heartbeat,
        );
        assert!(matches!(status_msg.message_type, MessageType::Status));
        assert!(status_msg.session_id.is_none());
    }
    
    #[test]
    fn test_key_modifiers_default() {
        let modifiers = KeyModifiers::default();
        assert!(!modifiers.shift);
        assert!(!modifiers.control);
        assert!(!modifiers.alt);
        assert!(!modifiers.super_key);
    }
    
    #[test]
    fn test_mouse_button_serialization() {
        let buttons = vec![
            MouseButton::Left,
            MouseButton::Right,
            MouseButton::Middle,
            MouseButton::X1,
            MouseButton::X2,
        ];
        
        for button in buttons {
            let json = serde_json::to_string(&button).unwrap();
            let deserialized: MouseButton = serde_json::from_str(&json).unwrap();
            assert_eq!(std::mem::discriminant(&button), std::mem::discriminant(&deserialized));
        }
    }
    
    #[test]
    fn test_keycode_serialization() {
        let keys = vec![
            KeyCode::A, KeyCode::B, KeyCode::Z,
            KeyCode::Key0, KeyCode::Key9,
            KeyCode::F1, KeyCode::F12,
            KeyCode::Space, KeyCode::Enter, KeyCode::Tab,
            KeyCode::ArrowUp, KeyCode::ArrowDown,
            KeyCode::LeftShift, KeyCode::RightControl,
        ];
        
        for key in keys {
            let json = serde_json::to_string(&key).unwrap();
            let deserialized: KeyCode = serde_json::from_str(&json).unwrap();
            assert_eq!(std::mem::discriminant(&key), std::mem::discriminant(&deserialized));
        }
    }
    
    #[test]
    fn test_client_type_serialization() {
        let types = vec![ClientType::HidClient, ClientType::Commander];
        
        for client_type in types {
            let json = serde_json::to_string(&client_type).unwrap();
            let deserialized: ClientType = serde_json::from_str(&json).unwrap();
            assert_eq!(std::mem::discriminant(&client_type), std::mem::discriminant(&deserialized));
        }
    }
    
    #[test]
    fn test_message_timestamp() {
        let before = Utc::now();
        let msg = Message::status(None, StatusMessage::Heartbeat);
        let after = Utc::now();
        
        assert!(msg.timestamp >= before);
        assert!(msg.timestamp <= after);
    }
}
