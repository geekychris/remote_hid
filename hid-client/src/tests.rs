#[cfg(test)]
mod tests {
    use super::*;
    use remote_hid_shared::*;
    use uuid::Uuid;
    
    #[test]
    fn test_hid_event_conversion() {
        // Test mouse move event
        let mouse_move = HidEvent::MouseMove { x: 100, y: 200, absolute: true };
        let json = serde_json::to_string(&mouse_move).unwrap();
        let deserialized: HidEvent = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            HidEvent::MouseMove { x, y, absolute } => {
                assert_eq!(x, 100);
                assert_eq!(y, 200);
                assert!(absolute);
            }
            _ => panic!("Wrong event type"),
        }
        
        // Test mouse click event
        let mouse_click = HidEvent::MouseClick {
            button: MouseButton::Left,
            pressed: true,
            x: Some(50),
            y: Some(75),
        };
        let json = serde_json::to_string(&mouse_click).unwrap();
        let deserialized: HidEvent = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            HidEvent::MouseClick { button, pressed, x, y } => {
                assert!(matches!(button, MouseButton::Left));
                assert!(pressed);
                assert_eq!(x, Some(50));
                assert_eq!(y, Some(75));
            }
            _ => panic!("Wrong event type"),
        }
        
        // Test keyboard event
        let key_event = HidEvent::KeyEvent {
            key: KeyCode::A,
            pressed: false,
            modifiers: KeyModifiers {
                shift: true,
                control: false,
                alt: false,
                super_key: false,
            },
        };
        let json = serde_json::to_string(&key_event).unwrap();
        let deserialized: HidEvent = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            HidEvent::KeyEvent { key, pressed, modifiers } => {
                assert!(matches!(key, KeyCode::A));
                assert!(!pressed);
                assert!(modifiers.shift);
                assert!(!modifiers.control);
            }
            _ => panic!("Wrong event type"),
        }
    }
    
    #[test]
    fn test_message_handling() {
        // Test HID event message
        let session_id = Uuid::new_v4();
        let hid_event = HidEvent::MouseMove { x: 10, y: 20, absolute: false };
        let message = Message::hid_event(session_id, hid_event);
        
        assert!(matches!(message.message_type, MessageType::HidEvent));
        assert_eq!(message.session_id, Some(session_id));
        
        // Test session control message
        let end_session = Message::session_control(
            Some(session_id),
            SessionControlMessage::EndSession,
        );
        
        assert!(matches!(end_session.message_type, MessageType::SessionControl));
        assert_eq!(end_session.session_id, Some(session_id));
        
        match end_session.payload {
            MessagePayload::SessionControl(SessionControlMessage::EndSession) => {
                // Expected
            }
            _ => panic!("Wrong message payload type"),
        }
    }
    
    #[test]
    fn test_session_ended_message() {
        let session_ended = SessionControlMessage::SessionEnded {
            reason: "Commander disconnected".to_string(),
        };
        
        let message = Message::session_control(None, session_ended);
        
        match message.payload {
            MessagePayload::SessionControl(SessionControlMessage::SessionEnded { reason }) => {
                assert_eq!(reason, "Commander disconnected");
            }
            _ => panic!("Wrong message type"),
        }
    }
    
    #[test]
    fn test_create_session_message() {
        let create_session = SessionControlMessage::CreateSession {
            client_id: "hid-client-123".to_string(),
            client_name: Some("Test HID Client".to_string()),
        };
        
        let message = Message::session_control(None, create_session);
        let json = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        
        match deserialized.payload {
            MessagePayload::SessionControl(SessionControlMessage::CreateSession { client_id, client_name }) => {
                assert_eq!(client_id, "hid-client-123");
                assert_eq!(client_name, Some("Test HID Client".to_string()));
            }
            _ => panic!("Wrong message type"),
        }
    }
    
    #[test]
    fn test_mouse_button_variants() {
        let buttons = vec![
            MouseButton::Left,
            MouseButton::Right,
            MouseButton::Middle,
            MouseButton::X1,
            MouseButton::X2,
        ];
        
        for button in buttons {
            let click_event = HidEvent::MouseClick {
                button,
                pressed: true,
                x: None,
                y: None,
            };
            
            // Test serialization
            let json = serde_json::to_string(&click_event).unwrap();
            let deserialized: HidEvent = serde_json::from_str(&json).unwrap();
            
            match deserialized {
                HidEvent::MouseClick { button: deserialized_button, .. } => {
                    assert_eq!(
                        std::mem::discriminant(&button),
                        std::mem::discriminant(&deserialized_button)
                    );
                }
                _ => panic!("Wrong event type"),
            }
        }
    }
    
    #[test]
    fn test_key_code_variants() {
        let keys = vec![
            KeyCode::A, KeyCode::B, KeyCode::C,
            KeyCode::Key0, KeyCode::Key1, KeyCode::Key9,
            KeyCode::F1, KeyCode::F12,
            KeyCode::Space, KeyCode::Enter, KeyCode::Tab,
            KeyCode::ArrowUp, KeyCode::ArrowDown,
            KeyCode::LeftShift, KeyCode::RightControl,
            KeyCode::Escape, KeyCode::Backspace,
        ];
        
        for key in keys {
            let key_event = HidEvent::KeyEvent {
                key,
                pressed: true,
                modifiers: KeyModifiers::default(),
            };
            
            // Test serialization
            let json = serde_json::to_string(&key_event).unwrap();
            let deserialized: HidEvent = serde_json::from_str(&json).unwrap();
            
            match deserialized {
                HidEvent::KeyEvent { key: deserialized_key, .. } => {
                    assert_eq!(
                        std::mem::discriminant(&key),
                        std::mem::discriminant(&deserialized_key)
                    );
                }
                _ => panic!("Wrong event type"),
            }
        }
    }
    
    #[test]
    fn test_mouse_scroll_event() {
        let scroll_event = HidEvent::MouseScroll {
            delta_x: -3,
            delta_y: 5,
            x: Some(100),
            y: Some(200),
        };
        
        let json = serde_json::to_string(&scroll_event).unwrap();
        let deserialized: HidEvent = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            HidEvent::MouseScroll { delta_x, delta_y, x, y } => {
                assert_eq!(delta_x, -3);
                assert_eq!(delta_y, 5);
                assert_eq!(x, Some(100));
                assert_eq!(y, Some(200));
            }
            _ => panic!("Wrong event type"),
        }
    }
    
    #[test]
    fn test_modifier_combinations() {
        let modifier_combinations = vec![
            KeyModifiers { shift: true, control: false, alt: false, super_key: false },
            KeyModifiers { shift: false, control: true, alt: false, super_key: false },
            KeyModifiers { shift: false, control: false, alt: true, super_key: false },
            KeyModifiers { shift: false, control: false, alt: false, super_key: true },
            KeyModifiers { shift: true, control: true, alt: false, super_key: false },
            KeyModifiers { shift: true, control: true, alt: true, super_key: true },
            KeyModifiers::default(),
        ];
        
        for modifiers in modifier_combinations {
            let key_event = HidEvent::KeyEvent {
                key: KeyCode::Space,
                pressed: true,
                modifiers: modifiers.clone(),
            };
            
            let json = serde_json::to_string(&key_event).unwrap();
            let deserialized: HidEvent = serde_json::from_str(&json).unwrap();
            
            match deserialized {
                HidEvent::KeyEvent { modifiers: deserialized_modifiers, .. } => {
                    assert_eq!(modifiers.shift, deserialized_modifiers.shift);
                    assert_eq!(modifiers.control, deserialized_modifiers.control);
                    assert_eq!(modifiers.alt, deserialized_modifiers.alt);
                    assert_eq!(modifiers.super_key, deserialized_modifiers.super_key);
                }
                _ => panic!("Wrong event type"),
            }
        }
    }
}

#[cfg(test)]
mod client_tests {
    use super::*;
    use remote_hid_shared::*;
    
    #[test]
    fn test_client_creation_parameters() {
        let server_url = "ws://127.0.0.1:8080".to_string();
        let client_id = "test-client".to_string();
        let client_name = Some("Test Client".to_string());
        
        // This would normally create a HidClient, but we can't easily test 
        // the full client without mocking the HID handler
        
        // Instead, test the message creation that would be used
        let create_session_msg = Message::session_control(
            None,
            SessionControlMessage::CreateSession {
                client_id: client_id.clone(),
                client_name: client_name.clone(),
            }
        );
        
        assert!(matches!(create_session_msg.message_type, MessageType::SessionControl));
        match create_session_msg.payload {
            MessagePayload::SessionControl(SessionControlMessage::CreateSession { 
                client_id: msg_client_id, 
                client_name: msg_client_name 
            }) => {
                assert_eq!(msg_client_id, client_id);
                assert_eq!(msg_client_name, client_name);
            }
            _ => panic!("Wrong message type"),
        }
    }
    
    #[test]
    fn test_message_deserialization() {
        let json_message = r#"{
            "message_type": {"type": "HidEvent"},
            "session_id": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": "2023-01-01T00:00:00Z",
            "payload": {
                "event_type": "MouseMove",
                "x": 100,
                "y": 200,
                "absolute": true
            }
        }"#;
        
        let result: std::result::Result<Message, _> = serde_json::from_str(json_message);
        assert!(result.is_ok());
        
        let message = result.unwrap();
        assert!(matches!(message.message_type, MessageType::HidEvent));
        assert!(message.session_id.is_some());
        
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
    fn test_invalid_message_handling() {
        let invalid_json = r#"{"invalid": "message"}"#;
        
        let result: std::result::Result<Message, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_status_message_handling() {
        let heartbeat_msg = Message::status(None, StatusMessage::Heartbeat);
        
        let json = serde_json::to_string(&heartbeat_msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        
        assert!(matches!(deserialized.message_type, MessageType::Status));
        match deserialized.payload {
            MessagePayload::Status(StatusMessage::Heartbeat) => {
                // Expected
            }
            _ => panic!("Wrong status message type"),
        }
    }
}

// Mock tests for platform-specific functionality
#[cfg(test)]
mod hid_handler_tests {
    use super::*;
    use remote_hid_shared::*;
    
    // These tests verify the event structure without actually executing HID operations
    
    #[test]
    fn test_hid_event_structure_validation() {
        // Test that all HID event types can be created and serialized
        let events = vec![
            HidEvent::MouseMove { x: 0, y: 0, absolute: true },
            HidEvent::MouseClick { 
                button: MouseButton::Left, 
                pressed: true, 
                x: None, 
                y: None 
            },
            HidEvent::MouseScroll { 
                delta_x: 0, 
                delta_y: 1, 
                x: None, 
                y: None 
            },
            HidEvent::KeyEvent { 
                key: KeyCode::Space, 
                pressed: true, 
                modifiers: KeyModifiers::default() 
            },
        ];
        
        for event in events {
            // Verify each event can be serialized and deserialized
            let json = serde_json::to_string(&event).unwrap();
            let deserialized: HidEvent = serde_json::from_str(&json).unwrap();
            
            // Verify the discriminant matches (same enum variant)
            assert_eq!(
                std::mem::discriminant(&event),
                std::mem::discriminant(&deserialized)
            );
        }
    }
    
    #[test]
    fn test_coordinate_ranges() {
        // Test extreme coordinate values
        let extreme_coords = vec![
            (0, 0),
            (i32::MAX, i32::MAX),
            (i32::MIN, i32::MIN),
            (-1000, -1000),
            (1920, 1080),  // Common screen resolution
            (4096, 2160),  // 4K resolution
        ];
        
        for (x, y) in extreme_coords {
            let mouse_move = HidEvent::MouseMove { x, y, absolute: true };
            let json = serde_json::to_string(&mouse_move).unwrap();
            let deserialized: HidEvent = serde_json::from_str(&json).unwrap();
            
            match deserialized {
                HidEvent::MouseMove { x: dx, y: dy, .. } => {
                    assert_eq!(x, dx);
                    assert_eq!(y, dy);
                }
                _ => panic!("Wrong event type"),
            }
        }
    }
    
    #[test]
    fn test_scroll_delta_ranges() {
        let scroll_deltas = vec![
            (0, 0),
            (1, 1),
            (-1, -1),
            (10, -10),
            (i32::MAX, i32::MIN),
        ];
        
        for (delta_x, delta_y) in scroll_deltas {
            let scroll_event = HidEvent::MouseScroll { 
                delta_x, 
                delta_y, 
                x: None, 
                y: None 
            };
            let json = serde_json::to_string(&scroll_event).unwrap();
            let deserialized: HidEvent = serde_json::from_str(&json).unwrap();
            
            match deserialized {
                HidEvent::MouseScroll { delta_x: dx, delta_y: dy, .. } => {
                    assert_eq!(delta_x, dx);
                    assert_eq!(delta_y, dy);
                }
                _ => panic!("Wrong event type"),
            }
        }
    }
}