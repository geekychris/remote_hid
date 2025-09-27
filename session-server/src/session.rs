use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use remote_hid_shared::ClientInfo;

/// Session state management
#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub commander_id: String,
    pub hid_client_id: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

impl Session {
    pub fn new(commander_id: String, hid_client_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            commander_id,
            hid_client_id,
            created_at: now,
            last_activity: now,
        }
    }
    
    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }
    
    pub fn is_expired(&self, timeout_mins: u64) -> bool {
        let timeout = chrono::Duration::minutes(timeout_mins as i64);
        Utc::now() - self.last_activity > timeout
    }
}

/// Session manager for tracking active sessions
#[derive(Debug, Default)]
pub struct SessionManager {
    sessions: HashMap<Uuid, Session>,
    client_sessions: HashMap<String, Uuid>, // client_id -> session_id
}

impl SessionManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn create_session(&mut self, commander_id: String, hid_client_id: String) -> Result<Uuid, String> {
        // Check if HID client is already in a session
        if self.client_sessions.contains_key(&hid_client_id) {
            return Err(format!("HID client {} is already in a session", hid_client_id));
        }
        
        let session = Session::new(commander_id, hid_client_id.clone());
        let session_id = session.id;
        
        self.sessions.insert(session_id, session);
        self.client_sessions.insert(hid_client_id, session_id);
        
        Ok(session_id)
    }
    
    pub fn end_session(&mut self, session_id: Uuid) -> Option<Session> {
        if let Some(session) = self.sessions.remove(&session_id) {
            self.client_sessions.remove(&session.hid_client_id);
            Some(session)
        } else {
            None
        }
    }
    
    pub fn get_session(&self, session_id: Uuid) -> Option<&Session> {
        self.sessions.get(&session_id)
    }
    
    pub fn get_session_by_client(&self, client_id: &str) -> Option<&Session> {
        self.client_sessions.get(client_id)
            .and_then(|&session_id| self.sessions.get(&session_id))
    }
    
    pub fn update_session_activity(&mut self, session_id: Uuid) {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.update_activity();
        }
    }
    
    pub fn cleanup_expired_sessions(&mut self, timeout_mins: u64) -> Vec<Session> {
        let mut expired = Vec::new();
        
        self.sessions.retain(|&session_id, session| {
            if session.is_expired(timeout_mins) {
                self.client_sessions.remove(&session.hid_client_id);
                expired.push(session.clone());
                false
            } else {
                true
            }
        });
        
        expired
    }
    
    pub fn list_sessions(&self) -> Vec<&Session> {
        self.sessions.values().collect()
    }
}