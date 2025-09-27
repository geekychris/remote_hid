#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::SessionManager;
    use remote_hid_shared::{Message, MessagePayload, MessageType, SessionControlMessage, HidEvent, MouseButton, KeyCode, KeyModifiers};
    use uuid::Uuid;
    use std::time::Duration;
    use chrono::{DateTime, Utc};

    #[test]
    fn test_session_manager_creation() {
        let manager = SessionManager::new();
        assert_eq!(manager.list_sessions().len(), 0);
    }

    #[test]
    fn test_session_creation() {
        let mut manager = SessionManager::new();
        
        let result = manager.create_session(
            "commander1".to_string(),
            "client1".to_string()
        );
        
        assert!(result.is_ok());
        let session_id = result.unwrap();
        
        // Verify session exists
        let session = manager.get_session(session_id);
        assert!(session.is_some());
        
        let session = session.unwrap();
        assert_eq!(session.commander_id, "commander1");
        assert_eq!(session.hid_client_id, "client1");
        
        // Verify session list
        let sessions = manager.list_sessions();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, session_id);
    }

    #[test]
    fn test_session_creation_duplicate_client() {
        let mut manager = SessionManager::new();
        
        // Create first session
        let result1 = manager.create_session(
            "commander1".to_string(),
            "client1".to_string()
        );
        assert!(result1.is_ok());
        
        // Try to create another session with same client
        let result2 = manager.create_session(
            "commander2".to_string(),
            "client1".to_string()
        );
        assert!(result2.is_err());
        
        // Should still only have one session
        assert_eq!(manager.list_sessions().len(), 1);
    }

    #[test]
    fn test_session_end() {
        let mut manager = SessionManager::new();
        
        let session_id = manager.create_session(
            "commander1".to_string(),
            "client1".to_string()
        ).unwrap();
        
        // End the session
        let ended_session = manager.end_session(session_id);
        assert!(ended_session.is_some());
        
        let ended = ended_session.unwrap();
        assert_eq!(ended.id, session_id);
        
        // Verify session is gone
        assert!(manager.get_session(session_id).is_none());
        assert_eq!(manager.list_sessions().len(), 0);
    }

    #[test]
    fn test_get_session_by_client() {
        let mut manager = SessionManager::new();
        
        let _session_id = manager.create_session(
            "commander1".to_string(),
            "client1".to_string()
        ).unwrap();
        
        // Find session by client ID
        let session = manager.get_session_by_client("client1");
        assert!(session.is_some());
        
        let session = session.unwrap();
        assert_eq!(session.hid_client_id, "client1");
        assert_eq!(session.commander_id, "commander1");
        
        // Non-existent client should return None
        assert!(manager.get_session_by_client("nonexistent").is_none());
    }

    #[test]
    fn test_session_activity_update() {
        let mut manager = SessionManager::new();
        
        let session_id = manager.create_session(
            "commander1".to_string(),
            "client1".to_string()
        ).unwrap();
        
        let original_activity = manager.get_session(session_id).unwrap().last_activity;
        
        // Sleep a tiny bit to ensure timestamp difference
        std::thread::sleep(Duration::from_millis(1));
        
        manager.update_session_activity(session_id);
        
        let updated_activity = manager.get_session(session_id).unwrap().last_activity;
        assert!(updated_activity > original_activity);
    }

    #[test]
    fn test_cleanup_expired_sessions() {
        let mut manager = SessionManager::new();
        
        let session_id = manager.create_session(
            "commander1".to_string(),
            "client1".to_string()
        ).unwrap();
        
        // No sessions should be expired with 1 minute timeout
        let expired = manager.cleanup_expired_sessions(1);
        assert_eq!(expired.len(), 0);
        assert_eq!(manager.list_sessions().len(), 1);
        
        // All sessions should be expired with 0 minute timeout
        let expired = manager.cleanup_expired_sessions(0);
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].id, session_id);
        assert_eq!(manager.list_sessions().len(), 0);
    }

    #[test]
    fn test_multiple_sessions() {
        let mut manager = SessionManager::new();
        
        let session1 = manager.create_session(
            "commander1".to_string(),
            "client1".to_string()
        ).unwrap();
        
        let session2 = manager.create_session(
            "commander2".to_string(),
            "client2".to_string()
        ).unwrap();
        
        let sessions = manager.list_sessions();
        assert_eq!(sessions.len(), 2);
        
        // Verify both sessions exist and are different
        assert_ne!(session1, session2);
        assert!(manager.get_session(session1).is_some());
        assert!(manager.get_session(session2).is_some());
    }
}

#[cfg(test)]
mod server_tests {
    use super::*;
use crate::config::{Config, ServerConfig, AuthConfig, SessionConfig};
    use std::collections::HashMap;
    
    fn create_test_config() -> Config {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                max_connections: 100,
                heartbeat_interval_secs: 30,
            },
            auth: AuthConfig {
                jwt_secret: "test_secret".to_string(),
                token_expiry_hours: 24,
                max_failed_attempts: 3,
                lockout_duration_mins: 15,
            },
            session: SessionConfig {
                max_sessions: 10,
                session_timeout_mins: 30,
                cleanup_interval_secs: 60,
            },
        }
    }

    #[tokio::test]
    async fn test_server_state_creation() {
        use crate::server::SessionServer;
        
        let config = create_test_config();
        let server = SessionServer::new(config).await;
        assert!(server.is_ok());
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use crate::config::Config;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_config_from_file() {
        let config_content = r#"
[server]
host = "0.0.0.0"
port = 9090
max_connections = 500
heartbeat_interval_secs = 60

[auth]
jwt_secret = "my_secret_key"
token_expiry_hours = 12
max_failed_attempts = 5
lockout_duration_mins = 30

[session]
max_sessions = 50
session_timeout_mins = 120
cleanup_interval_secs = 300
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();
        
        let config = Config::load(temp_file.path().to_str().unwrap());
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.server.max_connections, 500);
        assert_eq!(config.server.heartbeat_interval_secs, 60);
        assert_eq!(config.auth.jwt_secret, "my_secret_key");
        assert_eq!(config.auth.token_expiry_hours, 12);
        assert_eq!(config.auth.max_failed_attempts, 5);
        assert_eq!(config.auth.lockout_duration_mins, 30);
        assert_eq!(config.session.max_sessions, 50);
        assert_eq!(config.session.session_timeout_mins, 120);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.max_connections, 1000);
        assert_eq!(config.server.heartbeat_interval_secs, 30);
        assert_eq!(config.auth.max_failed_attempts, 3);
        assert_eq!(config.session.max_sessions, 100);
    }

    #[test]
    fn test_config_invalid_file() {
        let result = Config::load("nonexistent.toml");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod message_handling_tests {
    use super::*;
    use remote_hid_shared::*;
    use uuid::Uuid;

    #[test]
    fn test_hid_event_message_creation() {
        let session_id = Uuid::new_v4();
        let mouse_event = HidEvent::MouseMove { x: 100, y: 200, absolute: true };
        
        let message = Message::hid_event(session_id, mouse_event);
        
        assert!(matches!(message.message_type, MessageType::HidEvent));
        assert_eq!(message.session_id, Some(session_id));
        
        match message.payload {
            MessagePayload::HidEvent(HidEvent::MouseMove { x, y, absolute }) => {
                assert_eq!(x, 100);
                assert_eq!(y, 200);
                assert!(absolute);
            }
            _ => panic!("Wrong payload type"),
        }
    }

    #[test]
    fn test_session_control_message_creation() {
        let create_session = SessionControlMessage::CreateSession {
            client_id: "test_client".to_string(),
            client_name: Some("Test Client".to_string()),
        };
        
        let message = Message::session_control(None, create_session);
        
        assert!(matches!(message.message_type, MessageType::SessionControl));
        assert!(message.session_id.is_none());
        
        match message.payload {
            MessagePayload::SessionControl(SessionControlMessage::CreateSession { client_id, client_name }) => {
                assert_eq!(client_id, "test_client");
                assert_eq!(client_name, Some("Test Client".to_string()));
            }
            _ => panic!("Wrong payload type"),
        }
    }

    #[test]
    fn test_keyboard_event_message() {
        let session_id = Uuid::new_v4();
        let key_event = HidEvent::KeyEvent {
            key: KeyCode::Space,
            pressed: true,
            modifiers: KeyModifiers {
                shift: false,
                control: true,
                alt: false,
                super_key: false,
            },
        };
        
        let message = Message::hid_event(session_id, key_event);
        let json = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        
        match deserialized.payload {
            MessagePayload::HidEvent(HidEvent::KeyEvent { key, pressed, modifiers }) => {
                assert!(matches!(key, KeyCode::Space));
                assert!(pressed);
                assert!(!modifiers.shift);
                assert!(modifiers.control);
                assert!(!modifiers.alt);
                assert!(!modifiers.super_key);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_mouse_click_event_message() {
        let session_id = Uuid::new_v4();
        let click_event = HidEvent::MouseClick {
            button: MouseButton::Right,
            pressed: false,
            x: Some(150),
            y: Some(250),
        };
        
        let message = Message::hid_event(session_id, click_event);
        let json = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        
        match deserialized.payload {
            MessagePayload::HidEvent(HidEvent::MouseClick { button, pressed, x, y }) => {
                assert!(matches!(button, MouseButton::Right));
                assert!(!pressed);
                assert_eq!(x, Some(150));
                assert_eq!(y, Some(250));
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_status_message_types() {
        // Test heartbeat
        let heartbeat_msg = Message::status(None, StatusMessage::Heartbeat);
        assert!(matches!(heartbeat_msg.message_type, MessageType::Status));
        
        // Test connection status
        let status_msg = Message::status(
            Some(Uuid::new_v4()),
            StatusMessage::ConnectionStatus {
                connected: true,
                latency_ms: Some(25),
            }
        );
        
        let json = serde_json::to_string(&status_msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        
        match deserialized.payload {
            MessagePayload::Status(StatusMessage::ConnectionStatus { connected, latency_ms }) => {
                assert!(connected);
                assert_eq!(latency_ms, Some(25));
            }
            _ => panic!("Wrong status message type"),
        }
        
        // Test error message
        let error_msg = Message::status(
            None,
            StatusMessage::Error {
                error_code: "TIMEOUT".to_string(),
                error_message: "Connection timed out".to_string(),
            }
        );
        
        let json = serde_json::to_string(&error_msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        
        match deserialized.payload {
            MessagePayload::Status(StatusMessage::Error { error_code, error_message }) => {
                assert_eq!(error_code, "TIMEOUT");
                assert_eq!(error_message, "Connection timed out");
            }
            _ => panic!("Wrong status message type"),
        }
    }
}