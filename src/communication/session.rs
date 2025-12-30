//! USB Device Session Management
//!
//! Provides session-based device access with automatic cleanup and reconnection handling.

use super::DeviceHandle;
use crate::errors::UsbError;
use crate::model::UsbId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Session is active and device is accessible
    Active,
    /// Session is suspended (device may be in low power)
    Suspended,
    /// Session is disconnected (device removed)
    Disconnected,
    /// Session has an error
    Error,
}

/// Device session with automatic resource management
pub struct DeviceSession {
    /// Session ID
    pub id: u64,
    /// Device ID
    pub device_id: UsbId,
    /// Session creation time
    pub created_at: Instant,
    /// Last activity time
    pub last_activity: Instant,
    /// Session state
    pub state: SessionState,
    /// Device handle (if connected)
    handle: Option<DeviceHandle>,
    /// Claimed interfaces
    claimed_interfaces: Vec<u8>,
}

impl DeviceSession {
    /// Create a new session
    fn new(id: u64, handle: DeviceHandle) -> Self {
        let device_id = handle.id().clone();
        Self {
            id,
            device_id,
            created_at: Instant::now(),
            last_activity: Instant::now(),
            state: SessionState::Active,
            handle: Some(handle),
            claimed_interfaces: Vec::new(),
        }
    }
    
    /// Get the device handle
    pub fn handle(&self) -> Option<&DeviceHandle> {
        self.handle.as_ref()
    }
    
    /// Get mutable device handle
    pub fn handle_mut(&mut self) -> Option<&mut DeviceHandle> {
        self.handle.as_mut()
    }
    
    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }
    
    /// Get session duration
    pub fn duration(&self) -> Duration {
        self.created_at.elapsed()
    }
    
    /// Get idle duration
    pub fn idle_duration(&self) -> Duration {
        self.last_activity.elapsed()
    }
    
    /// Claim an interface
    pub fn claim_interface(&mut self, interface: u8) -> Result<(), UsbError> {
        if let Some(handle) = &mut self.handle {
            handle.claim_interface(interface)?;
            self.claimed_interfaces.push(interface);
            self.touch();
            Ok(())
        } else {
            Err(UsbError::DeviceNotFound("Session disconnected".into()))
        }
    }
    
    /// Release an interface
    pub fn release_interface(&mut self, interface: u8) -> Result<(), UsbError> {
        if let Some(handle) = &mut self.handle {
            handle.release_interface(interface)?;
            self.claimed_interfaces.retain(|&i| i != interface);
            self.touch();
            Ok(())
        } else {
            Err(UsbError::DeviceNotFound("Session disconnected".into()))
        }
    }
    
    /// Disconnect the session
    pub fn disconnect(&mut self) {
        self.handle = None;
        self.state = SessionState::Disconnected;
    }
    
    /// Check if session is active
    pub fn is_active(&self) -> bool {
        self.state == SessionState::Active && self.handle.is_some()
    }
}

/// Session manager for multiple devices
pub struct SessionManager {
    sessions: RwLock<HashMap<u64, Arc<Mutex<DeviceSession>>>>,
    next_id: Mutex<u64>,
    /// Session timeout (sessions idle longer than this are closed)
    idle_timeout: Duration,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            next_id: Mutex::new(1),
            idle_timeout: Duration::from_secs(300), // 5 minutes default
        }
    }
    
    /// Set the idle timeout
    pub fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }
    
    /// Open a new session for a device
    pub fn open(&self, vid: u16, pid: u16) -> Result<Arc<Mutex<DeviceSession>>, UsbError> {
        let handle = DeviceHandle::open(vid, pid)?;
        
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };
        
        let session = Arc::new(Mutex::new(DeviceSession::new(id, handle)));
        
        {
            let mut sessions = self.sessions.write().unwrap();
            sessions.insert(id, session.clone());
        }
        
        Ok(session)
    }
    
    /// Open a session by bus/address
    pub fn open_by_address(&self, bus: u8, address: u8) -> Result<Arc<Mutex<DeviceSession>>, UsbError> {
        let handle = DeviceHandle::open_by_address(bus, address)?;
        
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };
        
        let session = Arc::new(Mutex::new(DeviceSession::new(id, handle)));
        
        {
            let mut sessions = self.sessions.write().unwrap();
            sessions.insert(id, session.clone());
        }
        
        Ok(session)
    }
    
    /// Get a session by ID
    pub fn get(&self, id: u64) -> Option<Arc<Mutex<DeviceSession>>> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(&id).cloned()
    }
    
    /// Get a session by device ID
    pub fn get_by_device(&self, vid: u16, pid: u16) -> Option<Arc<Mutex<DeviceSession>>> {
        let sessions = self.sessions.read().unwrap();
        
        for session in sessions.values() {
            let session_lock = session.lock().unwrap();
            if session_lock.device_id.vid == vid && session_lock.device_id.pid == pid {
                if session_lock.is_active() {
                    return Some(session.clone());
                }
            }
        }
        
        None
    }
    
    /// Close a session
    pub fn close(&self, id: u64) {
        let mut sessions = self.sessions.write().unwrap();
        if let Some(session) = sessions.remove(&id) {
            let mut session_lock = session.lock().unwrap();
            session_lock.disconnect();
        }
    }
    
    /// Close all sessions
    pub fn close_all(&self) {
        let mut sessions = self.sessions.write().unwrap();
        for session in sessions.values() {
            let mut session_lock = session.lock().unwrap();
            session_lock.disconnect();
        }
        sessions.clear();
    }
    
    /// Clean up idle sessions
    pub fn cleanup_idle(&self) {
        let mut sessions = self.sessions.write().unwrap();
        let mut to_remove = Vec::new();
        
        for (&id, session) in sessions.iter() {
            let session_lock = session.lock().unwrap();
            if session_lock.idle_duration() > self.idle_timeout {
                to_remove.push(id);
            }
        }
        
        for id in to_remove {
            if let Some(session) = sessions.remove(&id) {
                let mut session_lock = session.lock().unwrap();
                session_lock.disconnect();
            }
        }
    }
    
    /// Get number of active sessions
    pub fn active_count(&self) -> usize {
        let sessions = self.sessions.read().unwrap();
        sessions.values()
            .filter(|s| s.lock().unwrap().is_active())
            .count()
    }
    
    /// Get all session IDs
    pub fn session_ids(&self) -> Vec<u64> {
        let sessions = self.sessions.read().unwrap();
        sessions.keys().copied().collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        self.close_all();
    }
}

/// RAII guard for automatic session cleanup
pub struct SessionGuard {
    manager: Arc<SessionManager>,
    session_id: u64,
}

impl SessionGuard {
    /// Create a new session guard
    pub fn new(manager: Arc<SessionManager>, session_id: u64) -> Self {
        Self { manager, session_id }
    }
    
    /// Get the session
    pub fn session(&self) -> Option<Arc<Mutex<DeviceSession>>> {
        self.manager.get(self.session_id)
    }
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        self.manager.close(self.session_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager() {
        let manager = SessionManager::new();
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_session_state() {
        assert_eq!(SessionState::Active, SessionState::Active);
        assert_ne!(SessionState::Active, SessionState::Disconnected);
    }
}
