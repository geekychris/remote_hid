#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_capture::{InputEvent, char_to_keycode};
    use remote_hid_shared::*;
    use uuid::Uuid;
    
    #[test]
    fn test_input_event_conversion() {
        // Test mouse move
        let input_mouse_move = InputEvent::MouseMove { x: 100, y: 200, absolute: true };
        match input_mouse_move {
            InputEvent::MouseMove { x, y, absolute } => {
                assert_eq!(x, 100);
                assert_eq!(y, 200);
                assert!(absolute);
            }
            _ => panic!("Wrong input event type"),
        }
        
        // Test mouse click
        let input_mouse_click = InputEvent::MouseClick {
            button: MouseButton::Right,
            pressed: false,
            x: Some(50),
            y: Some(75),
        };
        match input_mouse_click {
            InputEvent::MouseClick { button, pressed, x, y } => {
                assert!(matches!(button, MouseButton::Right));
                assert!(!pressed);
                assert_eq!(x, Some(50));
                assert_eq!(y, Some(75));
            }
            _ => panic!("Wrong input event type"),
        }
        
        // Test keyboard event
        let input_key = InputEvent::KeyEvent {
            key: KeyCode::Space,
            pressed: true,
            modifiers: KeyModifiers {
                shift: false,
                control: true,
                alt: false,
                super_key: false,
            },
        };
        match input_key {
            InputEvent::KeyEvent { key, pressed, modifiers } => {
                assert!(matches!(key, KeyCode::Space));
                assert!(pressed);
                assert!(!modifiers.shift);
                assert!(modifiers.control);
            }
            _ => panic!("Wrong input event type"),
        }
    }
    
    #[test]
    fn test_char_to_keycode_mapping() {
        // Test letters
        assert_eq!(char_to_keycode('A'), Some(KeyCode::A));
        assert_eq!(char_to_keycode('a'), Some(KeyCode::A)); // Should convert to uppercase
        assert_eq!(char_to_keycode('Z'), Some(KeyCode::Z));
        assert_eq!(char_to_keycode('z'), Some(KeyCode::Z));
        
        // Test numbers
        assert_eq!(char_to_keycode('0'), Some(KeyCode::Key0));
        assert_eq!(char_to_keycode('5'), Some(KeyCode::Key5));
        assert_eq!(char_to_keycode('9'), Some(KeyCode::Key9));
        
        // Test space
        assert_eq!(char_to_keycode(' '), Some(KeyCode::Space));
        
        // Test unsupported characters
        assert_eq!(char_to_keycode('@'), None);
        assert_eq!(char_to_keycode('!'), None);
        assert_eq!(char_to_keycode('€'), None);
    }
    
    #[test]
    fn test_input_to_hid_conversion() {
        // Create a mock commander for testing conversion logic
        struct TestCommander;
        
        impl TestCommander {
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
        
        let commander = TestCommander;
        
        // Test mouse move conversion
        let input_move = InputEvent::MouseMove { x: 300, y: 400, absolute: false };
        let hid_event = commander.convert_input_to_hid(input_move).unwrap();
        match hid_event {
            HidEvent::MouseMove { x, y, absolute } => {
                assert_eq!(x, 300);
                assert_eq!(y, 400);
                assert!(!absolute);
            }
            _ => panic!("Wrong HID event type"),
        }
        
        // Test mouse click conversion
        let input_click = InputEvent::MouseClick {
            button: MouseButton::Middle,
            pressed: true,
            x: None,
            y: None,
        };
        let hid_event = commander.convert_input_to_hid(input_click).unwrap();
        match hid_event {
            HidEvent::MouseClick { button, pressed, x, y } => {
                assert!(matches!(button, MouseButton::Middle));
                assert!(pressed);
                assert_eq!(x, None);
                assert_eq!(y, None);
            }
            _ => panic!("Wrong HID event type"),
        }
        
        // Test key event conversion
        let input_key = InputEvent::KeyEvent {
            key: KeyCode::Enter,
            pressed: false,
            modifiers: KeyModifiers {
                shift: true,
                control: false,
                alt: true,
                super_key: false,
            },
        };
        let hid_event = commander.convert_input_to_hid(input_key).unwrap();
        match hid_event {
            HidEvent::KeyEvent { key, pressed, modifiers } => {
                assert!(matches!(key, KeyCode::Enter));
                assert!(!pressed);
                assert!(modifiers.shift);
                assert!(!modifiers.control);
                assert!(modifiers.alt);
                assert!(!modifiers.super_key);
            }
            _ => panic!("Wrong HID event type"),
        }
    }
    
    #[test]
    fn test_join_session_message() {
        let target_client_id = "test_hid_client".to_string();
        
        let join_message = Message::session_control(
            None,
            SessionControlMessage::JoinSession {
                target_client_id: target_client_id.clone(),
            },
        );
        
        assert!(matches!(join_message.message_type, MessageType::SessionControl));
        assert!(join_message.session_id.is_none());
        
        match join_message.payload {
            MessagePayload::SessionControl(SessionControlMessage::JoinSession { target_client_id: msg_target }) => {
                assert_eq!(msg_target, target_client_id);
            }
            _ => panic!("Wrong message type"),
        }
    }
    
    #[test]
    fn test_hid_event_message_creation() {
        let session_id = Uuid::new_v4();
        
        // Test different HID event types in messages
        let events = vec![
            HidEvent::MouseMove { x: 10, y: 20, absolute: true },
            HidEvent::MouseClick { button: MouseButton::Left, pressed: true, x: None, y: None },
            HidEvent::MouseScroll { delta_x: -2, delta_y: 3, x: Some(100), y: Some(200) },
            HidEvent::KeyEvent { 
                key: KeyCode::Tab, 
                pressed: false, 
                modifiers: KeyModifiers::default() 
            },
        ];
        
        for event in events {
            let message = Message::hid_event(session_id, event.clone());
            
            assert!(matches!(message.message_type, MessageType::HidEvent));
            assert_eq!(message.session_id, Some(session_id));
            
            // Serialize and deserialize to test JSON compatibility
            let json = serde_json::to_string(&message).unwrap();
            let deserialized: Message = serde_json::from_str(&json).unwrap();
            
            assert!(matches!(deserialized.message_type, MessageType::HidEvent));
            assert_eq!(deserialized.session_id, Some(session_id));
            
            // Verify payload type matches
            match (&message.payload, &deserialized.payload) {
                (MessagePayload::HidEvent(original), MessagePayload::HidEvent(deserialized_event)) => {
                    assert_eq!(
                        std::mem::discriminant(original),
                        std::mem::discriminant(deserialized_event)
                    );
                }
                _ => panic!("Payload type mismatch"),
            }
        }
    }
    
    #[test]
    fn test_input_event_types() {
        // Test all InputEvent variants can be created
        let events = vec![
            InputEvent::MouseMove { x: 0, y: 0, absolute: true },
            InputEvent::MouseClick { 
                button: MouseButton::X1, 
                pressed: false, 
                x: Some(123), 
                y: Some(456) 
            },
            InputEvent::MouseScroll { 
                delta_x: -5, 
                delta_y: 10, 
                x: None, 
                y: None 
            },
            InputEvent::KeyEvent { 
                key: KeyCode::F5, 
                pressed: true, 
                modifiers: KeyModifiers { 
                    shift: false, 
                    control: true, 
                    alt: false, 
                    super_key: true 
                } 
            },
        ];
        
        // Verify all events can be pattern matched
        for event in events {
            match event {
                InputEvent::MouseMove { .. } => { /* OK */ }
                InputEvent::MouseClick { .. } => { /* OK */ }
                InputEvent::MouseScroll { .. } => { /* OK */ }
                InputEvent::KeyEvent { .. } => { /* OK */ }
            }
        }
    }
    
    #[test]
    fn test_all_mouse_buttons() {
        let buttons = vec![
            MouseButton::Left,
            MouseButton::Right, 
            MouseButton::Middle,
            MouseButton::X1,
            MouseButton::X2,
        ];
        
        for button in buttons {
            let click_event = InputEvent::MouseClick {
                button,
                pressed: true,
                x: Some(100),
                y: Some(200),
            };
            
            // Test that each button type can be used in input events
            match click_event {
                InputEvent::MouseClick { button: event_button, .. } => {
                    assert_eq!(
                        std::mem::discriminant(&button),
                        std::mem::discriminant(&event_button)
                    );
                }
                _ => panic!("Wrong event type"),
            }
        }
    }
    
    #[test]
    fn test_key_modifiers_combinations() {
        let modifier_sets = vec![
            KeyModifiers { shift: true, control: false, alt: false, super_key: false },
            KeyModifiers { shift: false, control: true, alt: false, super_key: false },
            KeyModifiers { shift: false, control: false, alt: true, super_key: false },
            KeyModifiers { shift: false, control: false, alt: false, super_key: true },
            KeyModifiers { shift: true, control: true, alt: false, super_key: false },
            KeyModifiers { shift: true, control: true, alt: true, super_key: true },
            KeyModifiers::default(), // All false
        ];
        
        for modifiers in modifier_sets {
            let key_event = InputEvent::KeyEvent {
                key: KeyCode::A,
                pressed: true,
                modifiers: modifiers.clone(),
            };
            
            match key_event {
                InputEvent::KeyEvent { modifiers: event_modifiers, .. } => {
                    assert_eq!(modifiers.shift, event_modifiers.shift);
                    assert_eq!(modifiers.control, event_modifiers.control);
                    assert_eq!(modifiers.alt, event_modifiers.alt);
                    assert_eq!(modifiers.super_key, event_modifiers.super_key);
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
    fn test_commander_parameters() {
        let server_url = "ws://127.0.0.1:8081".to_string();
        let target_client_id = "hid_client_123".to_string();
        
        // Test that Commander creation parameters are handled correctly
        // (We can't easily test the full Commander without WebSocket mocking)
        
        // Test the message that would be sent during connection
        let join_session_msg = Message::session_control(
            None,
            SessionControlMessage::JoinSession {
                target_client_id: target_client_id.clone(),
            }
        );
        
        let json = serde_json::to_string(&join_session_msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        
        match deserialized.payload {
            MessagePayload::SessionControl(SessionControlMessage::JoinSession { target_client_id: msg_target }) => {
                assert_eq!(msg_target, target_client_id);
            }
            _ => panic!("Wrong message type"),
        }
    }
    
    #[test]
    fn test_session_ended_handling() {
        let session_ended_msg = Message::session_control(
            None,
            SessionControlMessage::SessionEnded {
                reason: "Target client disconnected".to_string(),
            }
        );
        
        assert!(matches!(session_ended_msg.message_type, MessageType::SessionControl));
        
        match session_ended_msg.payload {
            MessagePayload::SessionControl(SessionControlMessage::SessionEnded { reason }) => {
                assert_eq!(reason, "Target client disconnected");
            }
            _ => panic!("Wrong message type"),
        }
    }
    
    #[test]
    fn test_status_message_handling() {
        // Test heartbeat status
        let heartbeat = Message::status(None, StatusMessage::Heartbeat);
        assert!(matches!(heartbeat.message_type, MessageType::Status));
        
        // Test connection status
        let conn_status = Message::status(
            None,
            StatusMessage::ConnectionStatus {
                connected: true,
                latency_ms: Some(15),
            }
        );
        
        match conn_status.payload {
            MessagePayload::Status(StatusMessage::ConnectionStatus { connected, latency_ms }) => {
                assert!(connected);
                assert_eq!(latency_ms, Some(15));
            }
            _ => panic!("Wrong status message type"),
        }
    }
}

#[cfg(test)]
mod input_capture_tests {
    use super::*;
    use crate::input_capture::{char_to_keycode, InputEvent};
    use remote_hid_shared::{KeyCode, KeyModifiers, MouseButton};
    
    #[test]
    fn test_char_to_keycode_comprehensive() {
        // Test all letters
        let letter_mappings = vec![
            ('a', KeyCode::A), ('A', KeyCode::A),
            ('b', KeyCode::B), ('B', KeyCode::B),
            ('c', KeyCode::C), ('C', KeyCode::C),
            ('d', KeyCode::D), ('D', KeyCode::D),
            ('e', KeyCode::E), ('E', KeyCode::E),
            ('f', KeyCode::F), ('F', KeyCode::F),
            ('g', KeyCode::G), ('G', KeyCode::G),
            ('h', KeyCode::H), ('H', KeyCode::H),
            ('i', KeyCode::I), ('I', KeyCode::I),
            ('j', KeyCode::J), ('J', KeyCode::J),
            ('k', KeyCode::K), ('K', KeyCode::K),
            ('l', KeyCode::L), ('L', KeyCode::L),
            ('m', KeyCode::M), ('M', KeyCode::M),
            ('n', KeyCode::N), ('N', KeyCode::N),
            ('o', KeyCode::O), ('O', KeyCode::O),
            ('p', KeyCode::P), ('P', KeyCode::P),
            ('q', KeyCode::Q), ('Q', KeyCode::Q),
            ('r', KeyCode::R), ('R', KeyCode::R),
            ('s', KeyCode::S), ('S', KeyCode::S),
            ('t', KeyCode::T), ('T', KeyCode::T),
            ('u', KeyCode::U), ('U', KeyCode::U),
            ('v', KeyCode::V), ('V', KeyCode::V),
            ('w', KeyCode::W), ('W', KeyCode::W),
            ('x', KeyCode::X), ('X', KeyCode::X),
            ('y', KeyCode::Y), ('Y', KeyCode::Y),
            ('z', KeyCode::Z), ('Z', KeyCode::Z),
        ];
        
        for (input_char, expected_key) in letter_mappings {
            assert_eq!(char_to_keycode(input_char), Some(expected_key));
        }
        
        // Test all numbers
        let number_mappings = vec![
            ('0', KeyCode::Key0),
            ('1', KeyCode::Key1),
            ('2', KeyCode::Key2),
            ('3', KeyCode::Key3),
            ('4', KeyCode::Key4),
            ('5', KeyCode::Key5),
            ('6', KeyCode::Key6),
            ('7', KeyCode::Key7),
            ('8', KeyCode::Key8),
            ('9', KeyCode::Key9),
        ];
        
        for (input_char, expected_key) in number_mappings {
            assert_eq!(char_to_keycode(input_char), Some(expected_key));
        }
        
        // Test space
        assert_eq!(char_to_keycode(' '), Some(KeyCode::Space));
        
        // Test unmapped characters
        let unmapped_chars = vec!['!', '@', '#', '$', '%', '^', '&', '*', '(', ')', 
                                 '-', '=', '[', ']', '\\', ';', '\'', ',', '.', '/', 
                                 '`', '~', '{', '}', '|', ':', '"', '<', '>', '?'];
        
        for unmapped_char in unmapped_chars {
            assert_eq!(char_to_keycode(unmapped_char), None);
        }
    }
    
    #[test]
    fn test_input_event_cloning() {
        let original_events = vec![
            InputEvent::MouseMove { x: 100, y: 200, absolute: true },
            InputEvent::MouseClick { 
                button: MouseButton::Right, 
                pressed: false, 
                x: Some(50), 
                y: Some(75) 
            },
            InputEvent::MouseScroll { 
                delta_x: -2, 
                delta_y: 3, 
                x: None, 
                y: None 
            },
            InputEvent::KeyEvent { 
                key: KeyCode::Space, 
                pressed: true, 
                modifiers: KeyModifiers::default() 
            },
        ];
        
        // Test that InputEvent implements Clone correctly
        for original in original_events {
            let cloned = original.clone();
            
            // Compare discriminants to ensure same variant
            assert_eq!(
                std::mem::discriminant(&original),
                std::mem::discriminant(&cloned)
            );
            
            // Test specific fields for each variant
            match (original, cloned) {
                (InputEvent::MouseMove { x: x1, y: y1, absolute: abs1 }, 
                 InputEvent::MouseMove { x: x2, y: y2, absolute: abs2 }) => {
                    assert_eq!(x1, x2);
                    assert_eq!(y1, y2);
                    assert_eq!(abs1, abs2);
                }
                (InputEvent::MouseClick { button: b1, pressed: p1, x: x1, y: y1 },
                 InputEvent::MouseClick { button: b2, pressed: p2, x: x2, y: y2 }) => {
                    assert_eq!(std::mem::discriminant(&b1), std::mem::discriminant(&b2));
                    assert_eq!(p1, p2);
                    assert_eq!(x1, x2);
                    assert_eq!(y1, y2);
                }
                (InputEvent::MouseScroll { delta_x: dx1, delta_y: dy1, x: x1, y: y1 },
                 InputEvent::MouseScroll { delta_x: dx2, delta_y: dy2, x: x2, y: y2 }) => {
                    assert_eq!(dx1, dx2);
                    assert_eq!(dy1, dy2);
                    assert_eq!(x1, x2);
                    assert_eq!(y1, y2);
                }
                (InputEvent::KeyEvent { key: k1, pressed: p1, modifiers: m1 },
                 InputEvent::KeyEvent { key: k2, pressed: p2, modifiers: m2 }) => {
                    assert_eq!(std::mem::discriminant(&k1), std::mem::discriminant(&k2));
                    assert_eq!(p1, p2);
                    assert_eq!(m1.shift, m2.shift);
                    assert_eq!(m1.control, m2.control);
                    assert_eq!(m1.alt, m2.alt);
                    assert_eq!(m1.super_key, m2.super_key);
                }
                _ => panic!("Cloned event has different variant than original"),
            }
        }
    }
}