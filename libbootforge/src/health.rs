//! Evidence-backed USB connection health reporting.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// High-level connection-health classification.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthState {
    Unknown,
    Healthy,
    Degraded,
    Unstable,
}

/// Observable evidence contributing to a health report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthSignal {
    EnumerationSucceeded,
    EnumerationFailure,
    Disconnect,
    Reconnect,
    RapidReconnect,
    ModeTransition,
}

/// Evidence-backed report for one stable device identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthReport {
    pub state: HealthState,
    pub score: Option<u8>,
    pub signals: Vec<HealthSignal>,
    pub enumeration_success_count: u32,
    pub enumeration_failure_count: u32,
    pub disconnect_count: u32,
    pub reconnect_count: u32,
    pub rapid_reconnect_count: u32,
    pub mode_transition_count: u32,
}

impl HealthReport {
    /// Compatibility constructor retained for existing consumers.
    pub fn from_counts(disconnect_count: u32, reconnect_count: u32, rapid_reconnects: u32) -> Self {
        Self::from_evidence(
            1,
            0,
            disconnect_count,
            reconnect_count,
            rapid_reconnects,
            0,
        )
    }

    /// Build a report only from observable counters.
    pub fn from_evidence(
        enumeration_success_count: u32,
        enumeration_failure_count: u32,
        disconnect_count: u32,
        reconnect_count: u32,
        rapid_reconnect_count: u32,
        mode_transition_count: u32,
    ) -> Self {
        if enumeration_success_count == 0
            && enumeration_failure_count == 0
            && disconnect_count == 0
            && reconnect_count == 0
            && rapid_reconnect_count == 0
            && mode_transition_count == 0
        {
            return Self::unknown();
        }

        let mut signals = Vec::new();
        if enumeration_success_count > 0 {
            signals.push(HealthSignal::EnumerationSucceeded);
        }
        if enumeration_failure_count > 0 {
            signals.push(HealthSignal::EnumerationFailure);
        }
        if disconnect_count > 0 {
            signals.push(HealthSignal::Disconnect);
        }
        if reconnect_count > 0 {
            signals.push(HealthSignal::Reconnect);
        }
        if rapid_reconnect_count > 0 {
            signals.push(HealthSignal::RapidReconnect);
        }
        if mode_transition_count > 0 {
            signals.push(HealthSignal::ModeTransition);
        }

        let penalty = enumeration_failure_count
            .saturating_mul(18)
            .saturating_add(disconnect_count.saturating_mul(7))
            .saturating_add(rapid_reconnect_count.saturating_mul(16))
            .saturating_add(mode_transition_count.saturating_mul(2))
            .min(100) as u8;
        let score = 100_u8.saturating_sub(penalty);
        let state = match score {
            85..=100 => HealthState::Healthy,
            60..=84 => HealthState::Degraded,
            _ => HealthState::Unstable,
        };

        Self {
            state,
            score: Some(score),
            signals,
            enumeration_success_count,
            enumeration_failure_count,
            disconnect_count,
            reconnect_count,
            rapid_reconnect_count,
            mode_transition_count,
        }
    }

    pub fn unknown() -> Self {
        Self {
            state: HealthState::Unknown,
            score: None,
            signals: Vec::new(),
            enumeration_success_count: 0,
            enumeration_failure_count: 0,
            disconnect_count: 0,
            reconnect_count: 0,
            rapid_reconnect_count: 0,
            mode_transition_count: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct HealthHistory {
    enumeration_success_count: u32,
    enumeration_failure_count: u32,
    disconnect_count: u32,
    reconnect_count: u32,
    rapid_reconnect_count: u32,
    mode_transition_count: u32,
    last_disconnect_at: Option<DateTime<Utc>>,
}

impl HealthHistory {
    fn report(&self) -> HealthReport {
        HealthReport::from_evidence(
            self.enumeration_success_count,
            self.enumeration_failure_count,
            self.disconnect_count,
            self.reconnect_count,
            self.rapid_reconnect_count,
            self.mode_transition_count,
        )
    }

    fn merge_from(&mut self, other: HealthHistory) {
        self.enumeration_success_count = self
            .enumeration_success_count
            .saturating_add(other.enumeration_success_count);
        self.enumeration_failure_count = self
            .enumeration_failure_count
            .saturating_add(other.enumeration_failure_count);
        self.disconnect_count = self.disconnect_count.saturating_add(other.disconnect_count);
        self.reconnect_count = self.reconnect_count.saturating_add(other.reconnect_count);
        self.rapid_reconnect_count = self
            .rapid_reconnect_count
            .saturating_add(other.rapid_reconnect_count);
        self.mode_transition_count = self
            .mode_transition_count
            .saturating_add(other.mode_transition_count);
        self.last_disconnect_at = match (self.last_disconnect_at, other.last_disconnect_at) {
            (Some(left), Some(right)) => Some(left.max(right)),
            (left @ Some(_), None) => left,
            (None, right) => right,
        };
    }
}

/// Stateful, in-memory health tracker keyed by stable device identity.
#[derive(Debug, Clone)]
pub struct HealthTracker {
    histories: HashMap<String, HealthHistory>,
    rapid_reconnect_window: Duration,
}

impl Default for HealthTracker {
    fn default() -> Self {
        Self::new(Duration::seconds(5))
    }
}

impl HealthTracker {
    pub fn new(rapid_reconnect_window: Duration) -> Self {
        Self {
            histories: HashMap::new(),
            rapid_reconnect_window,
        }
    }

    pub fn record_enumeration_success(&mut self, device_id: &str) -> HealthReport {
        let history = self.histories.entry(device_id.to_owned()).or_default();
        history.enumeration_success_count = history.enumeration_success_count.saturating_add(1);
        history.report()
    }

    pub fn record_enumeration_failure(&mut self, device_id: &str) -> HealthReport {
        let history = self.histories.entry(device_id.to_owned()).or_default();
        history.enumeration_failure_count = history.enumeration_failure_count.saturating_add(1);
        history.report()
    }

    pub fn record_disconnect(&mut self, device_id: &str, observed_at: DateTime<Utc>) -> HealthReport {
        let history = self.histories.entry(device_id.to_owned()).or_default();
        history.disconnect_count = history.disconnect_count.saturating_add(1);
        history.last_disconnect_at = Some(observed_at);
        history.report()
    }

    pub fn record_reconnect(&mut self, device_id: &str, observed_at: DateTime<Utc>) -> HealthReport {
        let history = self.histories.entry(device_id.to_owned()).or_default();
        history.reconnect_count = history.reconnect_count.saturating_add(1);
        if let Some(disconnected_at) = history.last_disconnect_at {
            let elapsed = observed_at.signed_duration_since(disconnected_at);
            if elapsed >= Duration::zero() && elapsed <= self.rapid_reconnect_window {
                history.rapid_reconnect_count = history.rapid_reconnect_count.saturating_add(1);
            }
        }
        history.report()
    }

    pub fn record_mode_transition(&mut self, device_id: &str) -> HealthReport {
        let history = self.histories.entry(device_id.to_owned()).or_default();
        history.mode_transition_count = history.mode_transition_count.saturating_add(1);
        history.report()
    }

    pub fn report(&self, device_id: &str) -> HealthReport {
        self.histories
            .get(device_id)
            .map(HealthHistory::report)
            .unwrap_or_else(HealthReport::unknown)
    }

    /// Move accumulated history when reconnect correlation proves an identity changed.
    pub fn merge_identity(&mut self, previous_id: &str, current_id: &str) {
        if previous_id == current_id {
            return;
        }
        if let Some(previous) = self.histories.remove(previous_id) {
            self.histories
                .entry(current_id.to_owned())
                .or_default()
                .merge_from(previous);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_rapid_reconnects_reduce_health() {
        let report = HealthReport::from_evidence(3, 0, 4, 4, 3, 0);
        assert_eq!(report.state, HealthState::Unstable);
        assert!(report.score.unwrap_or(100) < 60);
    }

    #[test]
    fn no_evidence_remains_unknown() {
        assert_eq!(HealthReport::unknown().state, HealthState::Unknown);
        assert_eq!(HealthReport::unknown().score, None);
    }

    #[test]
    fn tracker_detects_rapid_reconnect() {
        let mut tracker = HealthTracker::new(Duration::seconds(5));
        let start = Utc::now();
        tracker.record_enumeration_success("device-1");
        tracker.record_disconnect("device-1", start);
        let report = tracker.record_reconnect("device-1", start + Duration::seconds(2));
        assert_eq!(report.rapid_reconnect_count, 1);
        assert!(report.signals.contains(&HealthSignal::RapidReconnect));
    }

    #[test]
    fn identity_merge_preserves_history() {
        let mut tracker = HealthTracker::default();
        tracker.record_enumeration_success("old");
        tracker.record_disconnect("old", Utc::now());
        tracker.merge_identity("old", "new");
        let report = tracker.report("new");
        assert_eq!(report.enumeration_success_count, 1);
        assert_eq!(report.disconnect_count, 1);
    }
}
