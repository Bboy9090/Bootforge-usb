//! Pure recovery safety state machine: no device access and no write capability.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fault {
    Disconnect,
    Timeout,
    PartialRead,
    IdentityChanged,
    PowerLoss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryState {
    Ready,
    Observing,
    Aborted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoverySession {
    pub target_fingerprint: String,
    pub state: RecoveryState,
    pub observed_bytes: u64,
    pub abort_reason: Option<Fault>,
}

impl RecoverySession {
    pub fn new(fingerprint: impl Into<String>) -> Self {
        let target_fingerprint = fingerprint.into();
        let state = if target_fingerprint.trim().is_empty() {
            RecoveryState::Aborted
        } else {
            RecoveryState::Ready
        };
        Self {
            target_fingerprint,
            state,
            observed_bytes: 0,
            abort_reason: None,
        }
    }
    pub fn begin(&mut self, observed_fingerprint: &str) -> bool {
        if self.state != RecoveryState::Ready {
            return false;
        }
        if observed_fingerprint != self.target_fingerprint {
            self.fail(Fault::IdentityChanged);
            return false;
        }
        self.state = RecoveryState::Observing;
        true
    }
    pub fn record_read(&mut self, requested: usize, received: usize) -> bool {
        if self.state != RecoveryState::Observing {
            return false;
        }
        if received != requested {
            self.fail(Fault::PartialRead);
            return false;
        }
        match self.observed_bytes.checked_add(received as u64) {
            Some(next) => {
                self.observed_bytes = next;
                true
            }
            None => {
                self.fail(Fault::PartialRead);
                false
            }
        }
    }
    pub fn fail(&mut self, fault: Fault) {
        self.state = RecoveryState::Aborted;
        self.abort_reason = Some(fault);
    }
    pub fn can_continue(&self) -> bool {
        self.state == RecoveryState::Observing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid_observation_tracks_exact_bytes() {
        let mut s = RecoverySession::new("disk-A");
        assert!(s.begin("disk-A"));
        assert!(s.record_read(512, 512));
        assert_eq!(s.observed_bytes, 512);
    }
    #[test]
    fn wrong_device_fails_closed() {
        let mut s = RecoverySession::new("disk-A");
        assert!(!s.begin("disk-B"));
        assert_eq!(s.abort_reason, Some(Fault::IdentityChanged));
    }
    #[test]
    fn empty_identity_cannot_start() {
        let mut s = RecoverySession::new("");
        assert!(!s.begin(""));
    }
    #[test]
    fn partial_read_aborts_without_counting_bytes() {
        let mut s = RecoverySession::new("a");
        s.begin("a");
        assert!(!s.record_read(512, 256));
        assert_eq!(s.observed_bytes, 0);
        assert!(!s.can_continue());
    }
    #[test]
    fn disconnect_aborts() {
        let mut s = RecoverySession::new("a");
        s.begin("a");
        s.fail(Fault::Disconnect);
        assert!(!s.record_read(512, 512));
    }
    #[test]
    fn timeout_aborts() {
        let mut s = RecoverySession::new("a");
        s.begin("a");
        s.fail(Fault::Timeout);
        assert!(!s.can_continue());
    }
    #[test]
    fn power_loss_aborts() {
        let mut s = RecoverySession::new("a");
        s.begin("a");
        s.fail(Fault::PowerLoss);
        assert!(!s.can_continue());
    }
    #[test]
    fn abort_cannot_restart() {
        let mut s = RecoverySession::new("a");
        s.begin("a");
        s.fail(Fault::Disconnect);
        assert!(!s.begin("a"));
    }
}
