#[cfg(test)]
mod integration_tests {
    use crate::*;
    use uuid::Uuid;
    
    #[test]
    fn test_message_roundtrip_serialization() {
        let original_messages = vec![
            Message::auth_request(
                "testuser".to_string(),
                "password".to_string(),
                ClientType::Commander,
                Some("commander-1".to_string()),
            ),
            Message::hid_event(
                Uuid::new_v4(),
                HidEvent::MouseMove { x: 100, y: 200, absolute: true }
            ),
            Message::hid_event(
                Uuid::new_v4(),
                HidEvent::KeyEvent {
                    key: KeyCode::A,
                    pressed: true,
                    modifiers: KeyModifiers::default(),
                }
            ),
            Message::session_control(
                None,
                SessionControlMessage::CreateSession {
                    client_id: "hid-client-1".to_string(),
                    client_name: Some("Test Machine".to_string()),
                }
            ),
        ];
        
        for original in original_messages {
            let json = serde_json::to_string(&original).unwrap();
            let deserialized: Message = serde_json::from_str(&json).unwrap();
            
            // Verify message type matches
            assert_eq!(
                std::mem::discriminant(&original.message_type),
                std::mem::discriminant(&deserialized.message_type)
            );
            
            // Verify session ID matches
            assert_eq!(original.session_id, deserialized.session_id);
        }
    }
    
    #[test]
    fn test_keycode_coverage() {
        // Test that all common key codes can be serialized
        let test_keys = vec![
            KeyCode::A, KeyCode::Z,
            KeyCode::Key0, KeyCode::Key9,
            KeyCode::F1, KeyCode::F12,
            KeyCode::Space, KeyCode::Enter,
            KeyCode::ArrowUp, KeyCode::ArrowDown,
            KeyCode::LeftShift, KeyCode::RightControl,
        ];
        
        for key in test_keys {
            let event = HidEvent::KeyEvent {
                key,
                pressed: true,
                modifiers: KeyModifiers::default(),
            };
            
            let message = Message::hid_event(Uuid::new_v4(), event);
            let json = serde_json::to_string(&message).unwrap();
            let _deserialized: Message = serde_json::from_str(&json).unwrap();
        }
    }
    
    #[test]
    fn test_mouse_events() {
        let mouse_events = vec![
            HidEvent::MouseMove { x: 0, y: 0, absolute: true },
            HidEvent::MouseMove { x: -10, y: 5, absolute: false },
            HidEvent::MouseClick {
                button: MouseButton::Left,
                pressed: true,
                x: Some(100),
                y: Some(200),
            },
            HidEvent::MouseClick {
                button: MouseButton::Right,
                pressed: false,
                x: None,
                y: None,
            },
            HidEvent::MouseScroll {
                delta_x: 0,
                delta_y: 3,
                x: Some(150),
                y: Some(250),
            },
        ];
        
        for event in mouse_events {
            let message = Message::hid_event(Uuid::new_v4(), event);
            let json = serde_json::to_string(&message).unwrap();
            let _deserialized: Message = serde_json::from_str(&json).unwrap();
        }
    }
}