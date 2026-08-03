//! Evidence-backed USB connection health reporting.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthState {
    Unknown,
    Healthy,
    Degraded,
    Unstable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthSignal {
    EnumerationSucceeded,
    EnumerationFailure,
    Disconnect,
    Reconnect,
    RapidReconnect,
    ModeTransition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthReport {
    pub state: HealthState,
    pub score: Option<u8>,
    pub signals: Vec<HealthSignal>,
    pub disconnect_count: u32,
    pub reconnect_count: u32,
}

impl HealthReport {
    pub fn from_counts(disconnect_count: u32, reconnect_count: u32, rapid_reconnects: u32) -> Self {
        let mut signals = vec![HealthSignal::EnumerationSucceeded];
        signals.extend(std::iter::repeat(HealthSignal::Disconnect).take(disconnect_count as usize));
        signals.extend(std::iter::repeat(HealthSignal::Reconnect).take(reconnect_count as usize));
        signals.extend(std::iter::repeat(HealthSignal::RapidReconnect).take(rapid_reconnects as usize));

        let penalty = disconnect_count.saturating_mul(8)
            .saturating_add(rapid_reconnects.saturating_mul(15))
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
            disconnect_count,
            reconnect_count,
        }
    }

    pub fn unknown() -> Self {
        Self {
            state: HealthState::Unknown,
            score: None,
            signals: Vec::new(),
            disconnect_count: 0,
            reconnect_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_rapid_reconnects_reduce_health() {
        let report = HealthReport::from_counts(4, 4, 3);
        assert_eq!(report.state, HealthState::Unstable);
        assert!(report.score.unwrap_or(100) < 60);
    }

    #[test]
    fn no_instability_is_healthy() {
        let report = HealthReport::from_counts(0, 0, 0);
        assert_eq!(report.state, HealthState::Healthy);
        assert_eq!(report.score, Some(100));
    }
}
