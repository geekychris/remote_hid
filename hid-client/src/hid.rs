use anyhow::{Result, anyhow};
use remote_hid_shared::{HidEvent, MouseButton, KeyCode, KeyModifiers};
use tracing::{debug, warn};

// Platform-specific implementations are defined inline below

pub struct HidHandler {
    #[cfg(target_os = "macos")]
    inner: macos::MacOSHidHandler,
    #[cfg(target_os = "windows")]
    inner: windows::WindowsHidHandler,
}

impl HidHandler {
    pub fn new() -> Result<Self> {
        Ok(Self {
            #[cfg(target_os = "macos")]
            inner: macos::MacOSHidHandler::new()?,
            #[cfg(target_os = "windows")]
            inner: windows::WindowsHidHandler::new()?,
        })
    }
    
    pub async fn execute_event(&self, event: HidEvent) -> Result<()> {
        debug!("Executing HID event: {:?}", event);
        
        #[cfg(target_os = "macos")]
        return self.inner.execute_event(event).await;
        
        #[cfg(target_os = "windows")]
        return self.inner.execute_event(event).await;
        
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        return Err(anyhow!("Unsupported platform"));
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use core_graphics::event::{
        CGEvent, CGEventTapLocation, CGEventType, CGKeyCode, CGMouseButton
    };
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
    use core_graphics::geometry::{CGPoint, CGSize};
    use std::thread;
    use std::time::Duration;
    
    pub struct MacOSHidHandler {}
    
    impl MacOSHidHandler {
        pub fn new() -> Result<Self> {
            // Check for accessibility permissions
            if !Self::has_accessibility_permissions() {
                warn!("Accessibility permissions may be required for HID operations");
            }
            Ok(Self {})
        }
        
        fn has_accessibility_permissions() -> bool {
            // This is a simplified check - in production you'd want to check properly
            true
        }
        
        pub async fn execute_event(&self, event: HidEvent) -> Result<()> {
            // Execute on a separate thread to avoid blocking async runtime
            let result = tokio::task::spawn_blocking(move || {
                match event {
                    HidEvent::MouseMove { x, y, absolute } => {
                        Self::mouse_move(x, y, absolute)
                    }
                    HidEvent::MouseClick { button, pressed, x, y } => {
                        Self::mouse_click(button, pressed, x, y)
                    }
                    HidEvent::MouseScroll { delta_x, delta_y, x: _, y: _ } => {
                        Self::mouse_scroll(delta_x, delta_y)
                    }
                    HidEvent::KeyEvent { key, pressed, modifiers } => {
                        Self::key_event(key, pressed, modifiers)
                    }
                }
            }).await?;
            
            result
        }
        
        fn mouse_move(x: i32, y: i32, absolute: bool) -> Result<()> {
            let point = CGPoint::new(x as f64, y as f64);
            
            let event_type = if absolute {
                CGEventType::MouseMoved
            } else {
                // For relative movement, we need to get current position and add delta
                CGEventType::MouseMoved
            };
            
            let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
                .map_err(|_| anyhow!("Failed to create event source"))?;
            match CGEvent::new_mouse_event(
                source,
                event_type,
                point,
                CGMouseButton::Left, // Doesn't matter for move events
            ) {
                Ok(event) => {
                    event.post(CGEventTapLocation::HID);
                    debug!("Mouse moved to ({}, {})", x, y);
                }
                Err(_) => {
                    return Err(anyhow!("Failed to create mouse move event"));
                }
            }
            
            Ok(())
        }
        
        fn mouse_click(button: MouseButton, pressed: bool, x: Option<i32>, y: Option<i32>) -> Result<()> {
            let cg_button = match button {
                MouseButton::Left => CGMouseButton::Left,
                MouseButton::Right => CGMouseButton::Right,
                MouseButton::Middle => CGMouseButton::Center,
                _ => return Err(anyhow!("Unsupported mouse button: {:?}", button)),
            };
            
            let event_type = match (button, pressed) {
                (MouseButton::Left, true) => CGEventType::LeftMouseDown,
                (MouseButton::Left, false) => CGEventType::LeftMouseUp,
                (MouseButton::Right, true) => CGEventType::RightMouseDown,
                (MouseButton::Right, false) => CGEventType::RightMouseUp,
                (MouseButton::Middle, true) => CGEventType::OtherMouseDown,
                (MouseButton::Middle, false) => CGEventType::OtherMouseUp,
                _ => return Err(anyhow!("Unsupported mouse button combination")),
            };
            
            // Use current cursor position if x,y not provided
            let point = if let (Some(x), Some(y)) = (x, y) {
                CGPoint::new(x as f64, y as f64)
            } else {
                // Get current cursor position - simplified
                CGPoint::new(0.0, 0.0) // In production, get actual cursor position
            };
            
            let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
                .map_err(|_| anyhow!("Failed to create event source"))?;
            match CGEvent::new_mouse_event(source, event_type, point, cg_button) {
                Ok(event) => {
                    event.post(CGEventTapLocation::HID);
                    debug!("Mouse button {:?} {}", button, if pressed { "pressed" } else { "released" });
                }
                Err(_) => {
                    return Err(anyhow!("Failed to create mouse click event"));
                }
            }
            
            Ok(())
        }
        
        fn mouse_scroll(delta_x: i32, delta_y: i32) -> Result<()> {
            // Simplified scroll implementation - in production you'd want proper scroll events
            debug!("Mouse scroll requested ({}, {}) - simplified implementation", delta_x, delta_y);
            // Note: Core Graphics scroll wheel events are more complex to implement correctly
            // For now, we'll just log the scroll request
            // In a full implementation, you'd create proper scroll wheel events
            Ok(())
        }
        
        fn key_event(key: KeyCode, pressed: bool, _modifiers: KeyModifiers) -> Result<()> {
            let cg_keycode = Self::keycode_to_cg(key)?;
            let event_type = if pressed {
                CGEventType::KeyDown
            } else {
                CGEventType::KeyUp
            };
            
            let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
                .map_err(|_| anyhow!("Failed to create event source"))?;
            match CGEvent::new_keyboard_event(source, cg_keycode, pressed) {
                Ok(event) => {
                    event.post(CGEventTapLocation::HID);
                    debug!("Key {:?} {}", key, if pressed { "pressed" } else { "released" });
                }
                Err(_) => {
                    return Err(anyhow!("Failed to create keyboard event"));
                }
            }
            
            Ok(())
        }
        
        fn keycode_to_cg(key: KeyCode) -> Result<CGKeyCode> {
            // This is a simplified mapping - in production you'd want complete mapping
            let code = match key {
                KeyCode::A => 0,
                KeyCode::B => 11,
                KeyCode::C => 8,
                KeyCode::D => 2,
                KeyCode::E => 14,
                KeyCode::Space => 49,
                KeyCode::Enter => 36,
                KeyCode::Tab => 48,
                KeyCode::Escape => 53,
                KeyCode::ArrowUp => 126,
                KeyCode::ArrowDown => 125,
                KeyCode::ArrowLeft => 123,
                KeyCode::ArrowRight => 124,
                _ => {
                    warn!("Unmapped key code: {:?}, using default", key);
                    49 // Default to space
                }
            };
            
            Ok(code as CGKeyCode)
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use windows::{
        core::*,
        Win32::Foundation::*,
        Win32::UI::Input::KeyboardAndMouse::*,
        Win32::UI::WindowsAndMessaging::*,
    };
    
    pub struct WindowsHidHandler {}
    
    impl WindowsHidHandler {
        pub fn new() -> Result<Self> {
            Ok(Self {})
        }
        
        pub async fn execute_event(&self, event: HidEvent) -> Result<()> {
            // Execute on a separate thread to avoid blocking async runtime
            let result = tokio::task::spawn_blocking(move || {
                match event {
                    HidEvent::MouseMove { x, y, absolute } => {
                        Self::mouse_move(x, y, absolute)
                    }
                    HidEvent::MouseClick { button, pressed, x, y } => {
                        Self::mouse_click(button, pressed, x, y)
                    }
                    HidEvent::MouseScroll { delta_x, delta_y, x: _, y: _ } => {
                        Self::mouse_scroll(delta_x, delta_y)
                    }
                    HidEvent::KeyEvent { key, pressed, modifiers } => {
                        Self::key_event(key, pressed, modifiers)
                    }
                }
            }).await?;
            
            result
        }
        
        fn mouse_move(x: i32, y: i32, absolute: bool) -> Result<()> {
            unsafe {
                let mut input = INPUT::default();
                input.r#type = INPUT_MOUSE;
                input.Anonymous.mi = MOUSEINPUT {
                    dx: x,
                    dy: y,
                    mouseData: 0,
                    dwFlags: if absolute {
                        MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE
                    } else {
                        MOUSEEVENTF_MOVE
                    },
                    time: 0,
                    dwExtraInfo: 0,
                };
                
                let result = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                if result == 0 {
                    return Err(anyhow!("Failed to send mouse move input"));
                }
                
                debug!("Mouse moved to ({}, {})", x, y);
            }
            
            Ok(())
        }
        
        fn mouse_click(button: MouseButton, pressed: bool, x: Option<i32>, y: Option<i32>) -> Result<()> {
            unsafe {
                // Move to position if specified
                if let (Some(x), Some(y)) = (x, y) {
                    Self::mouse_move(x, y, true)?;
                }
                
                let flags = match (button, pressed) {
                    (MouseButton::Left, true) => MOUSEEVENTF_LEFTDOWN,
                    (MouseButton::Left, false) => MOUSEEVENTF_LEFTUP,
                    (MouseButton::Right, true) => MOUSEEVENTF_RIGHTDOWN,
                    (MouseButton::Right, false) => MOUSEEVENTF_RIGHTUP,
                    (MouseButton::Middle, true) => MOUSEEVENTF_MIDDLEDOWN,
                    (MouseButton::Middle, false) => MOUSEEVENTF_MIDDLEUP,
                    _ => return Err(anyhow!("Unsupported mouse button: {:?}", button)),
                };
                
                let mut input = INPUT::default();
                input.r#type = INPUT_MOUSE;
                input.Anonymous.mi = MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                };
                
                let result = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                if result == 0 {
                    return Err(anyhow!("Failed to send mouse click input"));
                }
                
                debug!("Mouse button {:?} {}", button, if pressed { "pressed" } else { "released" });
            }
            
            Ok(())
        }
        
        fn mouse_scroll(delta_x: i32, delta_y: i32) -> Result<()> {
            unsafe {
                // Vertical scroll
                if delta_y != 0 {
                    let mut input = INPUT::default();
                    input.r#type = INPUT_MOUSE;
                    input.Anonymous.mi = MOUSEINPUT {
                        dx: 0,
                        dy: 0,
                        mouseData: (delta_y * 120) as u32, // Windows uses 120 units per scroll
                        dwFlags: MOUSEEVENTF_WHEEL,
                        time: 0,
                        dwExtraInfo: 0,
                    };
                    
                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
                
                // Horizontal scroll
                if delta_x != 0 {
                    let mut input = INPUT::default();
                    input.r#type = INPUT_MOUSE;
                    input.Anonymous.mi = MOUSEINPUT {
                        dx: 0,
                        dy: 0,
                        mouseData: (delta_x * 120) as u32,
                        dwFlags: MOUSEEVENTF_HWHEEL,
                        time: 0,
                        dwExtraInfo: 0,
                    };
                    
                    SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
                
                debug!("Mouse scrolled ({}, {})", delta_x, delta_y);
            }
            
            Ok(())
        }
        
        fn key_event(key: KeyCode, pressed: bool, _modifiers: KeyModifiers) -> Result<()> {
            unsafe {
                let vk_code = Self::keycode_to_vk(key)?;
                
                let mut input = INPUT::default();
                input.r#type = INPUT_KEYBOARD;
                input.Anonymous.ki = KEYBDINPUT {
                    wVk: vk_code,
                    wScan: 0,
                    dwFlags: if pressed { KEYEVENTF_KEYDOWN } else { KEYEVENTF_KEYUP },
                    time: 0,
                    dwExtraInfo: 0,
                };
                
                let result = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                if result == 0 {
                    return Err(anyhow!("Failed to send keyboard input"));
                }
                
                debug!("Key {:?} {}", key, if pressed { "pressed" } else { "released" });
            }
            
            Ok(())
        }
        
        fn keycode_to_vk(key: KeyCode) -> Result<VIRTUAL_KEY> {
            let vk = match key {
                KeyCode::A => VK_A,
                KeyCode::B => VK_B,
                KeyCode::C => VK_C,
                KeyCode::D => VK_D,
                KeyCode::E => VK_E,
                KeyCode::Space => VK_SPACE,
                KeyCode::Enter => VK_RETURN,
                KeyCode::Tab => VK_TAB,
                KeyCode::Escape => VK_ESCAPE,
                KeyCode::ArrowUp => VK_UP,
                KeyCode::ArrowDown => VK_DOWN,
                KeyCode::ArrowLeft => VK_LEFT,
                KeyCode::ArrowRight => VK_RIGHT,
                _ => {
                    warn!("Unmapped key code: {:?}, using default", key);
                    VK_SPACE // Default to space
                }
            };
            
            Ok(vk)
        }
    }
}

// Stub implementation for unsupported platforms
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported {
    use super::*;
    
    pub struct UnsupportedHidHandler {}
    
    impl UnsupportedHidHandler {
        pub fn new() -> Result<Self> {
            Err(anyhow!("HID operations not supported on this platform"))
        }
        
        pub async fn execute_event(&self, _event: HidEvent) -> Result<()> {
            Err(anyhow!("HID operations not supported on this platform"))
        }
    }
}