use remote_hid_shared::*;
use serde_json;
use uuid::Uuid;
use std::time::Duration;
use chrono::Utc;

/// Integration tests for the Remote HID system
/// These tests verify end-to-end message flow and protocol compatibility

#[test]
fn test_full_message_protocol_compatibility() {
    // Test that all message types can be serialized and deserialized correctly
    // This simulates the full protocol flow between components
    
    let session_id = Uuid::new_v4();
    
    // 1. Test HID Client registration message
    let create_session_msg = Message::session_control(
        None,
        SessionControlMessage::CreateSession {
            client_id: "integration_test_client".to_string(),
            client_name: Some("Integration Test HID Client".to_string()),
        }
    );
    
    let json = serde_json::to_string(&create_session_msg).unwrap();
    let deserialized: Message = serde_json::from_str(&json).unwrap();
    assert!(matches!(deserialized.message_type, MessageType::SessionControl));
    
    // 2. Test Commander join session message
    let join_session_msg = Message::session_control(
        None,
        SessionControlMessage::JoinSession {
            target_client_id: "integration_test_client".to_string(),
        }
    );
    
    let json = serde_json::to_string(&join_session_msg).unwrap();
    let deserialized: Message = serde_json::from_str(&json).unwrap();
    assert!(matches!(deserialized.message_type, MessageType::SessionControl));
    
    // 3. Test various HID events that would flow from Commander to HID Client
    let hid_events = vec![
        HidEvent::MouseMove { x: 100, y: 200, absolute: true },
        HidEvent::MouseClick { 
            button: MouseButton::Left, 
            pressed: true, 
            x: Some(100), 
            y: Some(200) 
        },
        HidEvent::MouseClick { 
            button: MouseButton::Left, 
            pressed: false, 
            x: Some(100), 
            y: Some(200) 
        },
        HidEvent::MouseScroll { 
            delta_x: 0, 
            delta_y: -3, 
            x: Some(100), 
            y: Some(200) 
        },
        HidEvent::KeyEvent { 
            key: KeyCode::H, 
            pressed: true, 
            modifiers: KeyModifiers::default() 
        },
        HidEvent::KeyEvent { 
            key: KeyCode::H, 
            pressed: false, 
            modifiers: KeyModifiers::default() 
        },
        HidEvent::KeyEvent { 
            key: KeyCode::E, 
            pressed: true, 
            modifiers: KeyModifiers::default() 
        },
        HidEvent::KeyEvent { 
            key: KeyCode::E, 
            pressed: false, 
            modifiers: KeyModifiers::default() 
        },
    ];
    
    for event in hid_events {
        let hid_msg = Message::hid_event(session_id, event);
        let json = serde_json::to_string(&hid_msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        
        assert!(matches!(deserialized.message_type, MessageType::HidEvent));
        assert_eq!(deserialized.session_id, Some(session_id));
    }
    
    // 4. Test session termination
    let end_session_msg = Message::session_control(
        Some(session_id),
        SessionControlMessage::EndSession
    );
    
    let json = serde_json::to_string(&end_session_msg).unwrap();
    let deserialized: Message = serde_json::from_str(&json).unwrap();
    assert!(matches!(deserialized.message_type, MessageType::SessionControl));
    assert_eq!(deserialized.session_id, Some(session_id));
}

#[test]
fn test_message_timestamp_ordering() {
    // Test that message timestamps are properly ordered
    let mut messages = Vec::new();
    
    // Create messages with slight delays to ensure different timestamps
    for i in 0..5 {
        std::thread::sleep(Duration::from_millis(1));
        let msg = Message::status(None, StatusMessage::Heartbeat);
        messages.push(msg);
    }
    
    // Verify timestamps are in ascending order
    for window in messages.windows(2) {
        assert!(window[0].timestamp <= window[1].timestamp);
    }
}

#[test]
fn test_complex_key_combination_scenario() {
    // Test a complex typing scenario: "Hello World!" with modifiers
    let session_id = Uuid::new_v4();
    
    let typing_sequence = vec![
        // Shift + H (capital H)
        HidEvent::KeyEvent { 
            key: KeyCode::LeftShift, 
            pressed: true, 
            modifiers: KeyModifiers { shift: true, ..Default::default() } 
        },
        HidEvent::KeyEvent { 
            key: KeyCode::H, 
            pressed: true, 
            modifiers: KeyModifiers { shift: true, ..Default::default() } 
        },
        HidEvent::KeyEvent { 
            key: KeyCode::H, 
            pressed: false, 
            modifiers: KeyModifiers { shift: true, ..Default::default() } 
        },
        HidEvent::KeyEvent { 
            key: KeyCode::LeftShift, 
            pressed: false, 
            modifiers: KeyModifiers::default() 
        },
        
        // e, l, l, o
        HidEvent::KeyEvent { key: KeyCode::E, pressed: true, modifiers: KeyModifiers::default() },
        HidEvent::KeyEvent { key: KeyCode::E, pressed: false, modifiers: KeyModifiers::default() },
        HidEvent::KeyEvent { key: KeyCode::L, pressed: true, modifiers: KeyModifiers::default() },
        HidEvent::KeyEvent { key: KeyCode::L, pressed: false, modifiers: KeyModifiers::default() },
        HidEvent::KeyEvent { key: KeyCode::L, pressed: true, modifiers: KeyModifiers::default() },
        HidEvent::KeyEvent { key: KeyCode::L, pressed: false, modifiers: KeyModifiers::default() },
        HidEvent::KeyEvent { key: KeyCode::O, pressed: true, modifiers: KeyModifiers::default() },
        HidEvent::KeyEvent { key: KeyCode::O, pressed: false, modifiers: KeyModifiers::default() },
        
        // Space
        HidEvent::KeyEvent { key: KeyCode::Space, pressed: true, modifiers: KeyModifiers::default() },
        HidEvent::KeyEvent { key: KeyCode::Space, pressed: false, modifiers: KeyModifiers::default() },
        
        // Ctrl+A (Select All)
        HidEvent::KeyEvent { 
            key: KeyCode::LeftControl, 
            pressed: true, 
            modifiers: KeyModifiers { control: true, ..Default::default() } 
        },
        HidEvent::KeyEvent { 
            key: KeyCode::A, 
            pressed: true, 
            modifiers: KeyModifiers { control: true, ..Default::default() } 
        },
        HidEvent::KeyEvent { 
            key: KeyCode::A, 
            pressed: false, 
            modifiers: KeyModifiers { control: true, ..Default::default() } 
        },
        HidEvent::KeyEvent { 
            key: KeyCode::LeftControl, 
            pressed: false, 
            modifiers: KeyModifiers::default() 
        },
    ];
    
    // Test that all events in the sequence can be properly serialized
    for (i, event) in typing_sequence.into_iter().enumerate() {
        let message = Message::hid_event(session_id, event);
        let json = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        
        assert!(matches!(deserialized.message_type, MessageType::HidEvent));
        assert_eq!(deserialized.session_id, Some(session_id));
        
        // Verify the message has a proper timestamp
        assert!(deserialized.timestamp <= Utc::now());
        
        println!("Sequence step {}: Event serialized successfully", i + 1);
    }
}

#[test]
fn test_mouse_interaction_scenario() {
    // Test a complex mouse interaction: drag and drop operation
    let session_id = Uuid::new_v4();
    
    let drag_drop_sequence = vec![
        // Move to start position
        HidEvent::MouseMove { x: 100, y: 100, absolute: true },
        
        // Press left button (start drag)
        HidEvent::MouseClick { 
            button: MouseButton::Left, 
            pressed: true, 
            x: Some(100), 
            y: Some(100) 
        },
        
        // Drag to multiple intermediate positions
        HidEvent::MouseMove { x: 110, y: 105, absolute: true },
        HidEvent::MouseMove { x: 120, y: 110, absolute: true },
        HidEvent::MouseMove { x: 150, y: 130, absolute: true },
        HidEvent::MouseMove { x: 200, y: 150, absolute: true },
        
        // Release at final position (end drag)
        HidEvent::MouseClick { 
            button: MouseButton::Left, 
            pressed: false, 
            x: Some(200), 
            y: Some(150) 
        },
        
        // Right click for context menu
        HidEvent::MouseClick { 
            button: MouseButton::Right, 
            pressed: true, 
            x: Some(200), 
            y: Some(150) 
        },
        HidEvent::MouseClick { 
            button: MouseButton::Right, 
            pressed: false, 
            x: Some(200), 
            y: Some(150) 
        },
        
        // Scroll wheel interaction
        HidEvent::MouseScroll { 
            delta_x: 0, 
            delta_y: 3, 
            x: Some(200), 
            y: Some(150) 
        },
        HidEvent::MouseScroll { 
            delta_x: 0, 
            delta_y: -2, 
            x: Some(200), 
            y: Some(150) 
        },
    ];
    
    // Test that all mouse events can be properly handled
    for (i, event) in drag_drop_sequence.into_iter().enumerate() {
        let message = Message::hid_event(session_id, event);
        let json = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        
        assert!(matches!(deserialized.message_type, MessageType::HidEvent));
        assert_eq!(deserialized.session_id, Some(session_id));
        
        println!("Mouse sequence step {}: Event processed successfully", i + 1);
    }
}

#[test]
fn test_session_lifecycle_flow() {
    // Test the complete session lifecycle
    let client_id = "lifecycle_test_client".to_string();
    let session_id = Uuid::new_v4();
    
    // 1. HID Client connects and creates session
    let create_msg = Message::session_control(
        None,
        SessionControlMessage::CreateSession {
            client_id: client_id.clone(),
            client_name: Some("Lifecycle Test Client".to_string()),
        }
    );
    
    let json = serde_json::to_string(&create_msg).unwrap();
    let _: Message = serde_json::from_str(&json).unwrap();
    
    // 2. Commander joins the session
    let join_msg = Message::session_control(
        None,
        SessionControlMessage::JoinSession {
            target_client_id: client_id.clone(),
        }
    );
    
    let json = serde_json::to_string(&join_msg).unwrap();
    let _: Message = serde_json::from_str(&json).unwrap();
    
    // 3. Send some HID events during active session
    let test_events = vec![
        HidEvent::KeyEvent { 
            key: KeyCode::T, 
            pressed: true, 
            modifiers: KeyModifiers::default() 
        },
        HidEvent::KeyEvent { 
            key: KeyCode::T, 
            pressed: false, 
            modifiers: KeyModifiers::default() 
        },
        HidEvent::MouseMove { x: 50, y: 50, absolute: true },
    ];
    
    for event in test_events {
        let hid_msg = Message::hid_event(session_id, event);
        let json = serde_json::to_string(&hid_msg).unwrap();
        let _: Message = serde_json::from_str(&json).unwrap();
    }
    
    // 4. End the session
    let end_msg = Message::session_control(
        Some(session_id),
        SessionControlMessage::EndSession
    );
    
    let json = serde_json::to_string(&end_msg).unwrap();
    let _: Message = serde_json::from_str(&json).unwrap();
    
    // 5. Session ended notification
    let ended_msg = Message::session_control(
        Some(session_id),
        SessionControlMessage::SessionEnded {
            reason: "Session completed normally".to_string(),
        }
    );
    
    let json = serde_json::to_string(&ended_msg).unwrap();
    let deserialized: Message = serde_json::from_str(&json).unwrap();
    
    match deserialized.payload {
        MessagePayload::SessionControl(SessionControlMessage::SessionEnded { reason }) => {
            assert_eq!(reason, "Session completed normally");
        }
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn test_error_handling_scenarios() {
    // Test various error conditions and status messages
    let error_scenarios = vec![
        StatusMessage::Error {
            error_code: "INVALID_SESSION".to_string(),
            error_message: "Session ID does not exist".to_string(),
        },
        StatusMessage::Error {
            error_code: "CLIENT_DISCONNECTED".to_string(),
            error_message: "Target HID client has disconnected".to_string(),
        },
        StatusMessage::Error {
            error_code: "PERMISSION_DENIED".to_string(),
            error_message: "Insufficient permissions for HID operations".to_string(),
        },
        StatusMessage::ConnectionStatus {
            connected: false,
            latency_ms: None,
        },
        StatusMessage::ConnectionStatus {
            connected: true,
            latency_ms: Some(150), // High latency
        },
    ];
    
    for error_status in error_scenarios {
        let status_msg = Message::status(None, error_status);
        let json = serde_json::to_string(&status_msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        
        assert!(matches!(deserialized.message_type, MessageType::Status));
    }
}

#[test]
fn test_concurrent_session_handling() {
    // Test multiple concurrent sessions (at the protocol level)
    let client_ids = vec!["client_1", "client_2", "client_3"];
    let session_ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
    
    // Create multiple sessions
    for (i, client_id) in client_ids.iter().enumerate() {
        let create_msg = Message::session_control(
            None,
            SessionControlMessage::CreateSession {
                client_id: client_id.to_string(),
                client_name: Some(format!("Test Client {}", i + 1)),
            }
        );
        
        let json = serde_json::to_string(&create_msg).unwrap();
        let _: Message = serde_json::from_str(&json).unwrap();
        
        // Send events to each session
        let test_event = HidEvent::KeyEvent {
            key: KeyCode::A,
            pressed: true,
            modifiers: KeyModifiers::default(),
        };
        
        let hid_msg = Message::hid_event(session_ids[i], test_event);
        let json = serde_json::to_string(&hid_msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.session_id, Some(session_ids[i]));
    }
}

#[test]
fn test_large_coordinate_values() {
    // Test handling of extreme coordinate values
    let session_id = Uuid::new_v4();
    
    let extreme_coordinates = vec![
        (0, 0),
        (i32::MIN, i32::MIN),
        (i32::MAX, i32::MAX),
        (-1920, -1080),
        (3840, 2160), // 4K resolution
        (7680, 4320), // 8K resolution
    ];
    
    for (x, y) in extreme_coordinates {
        let mouse_event = HidEvent::MouseMove { x, y, absolute: true };
        let message = Message::hid_event(session_id, mouse_event);
        
        let json = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        
        match deserialized.payload {
            MessagePayload::HidEvent(HidEvent::MouseMove { x: dx, y: dy, .. }) => {
                assert_eq!(x, dx);
                assert_eq!(y, dy);
            }
            _ => panic!("Wrong event type"),
        }
    }
}

#[test]
fn test_json_message_size_limits() {
    // Test that complex messages don't exceed reasonable size limits
    let session_id = Uuid::new_v4();
    
    // Create a complex message with all optional fields filled
    let complex_event = HidEvent::MouseClick {
        button: MouseButton::Middle,
        pressed: true,
        x: Some(i32::MAX),
        y: Some(i32::MAX),
    };
    
    let message = Message::hid_event(session_id, complex_event);
    let json = serde_json::to_string(&message).unwrap();
    
    // Verify the JSON is reasonable in size (should be well under 1KB for individual events)
    assert!(json.len() < 1024, "Message JSON too large: {} bytes", json.len());
    
    // Verify it can be deserialized
    let _: Message = serde_json::from_str(&json).unwrap();
}