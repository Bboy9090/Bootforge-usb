//! Persistent per-device lifetime accounting driven by normalized forensic events.

use crate::{ForensicEvent, ForensicEventKind};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceLifetime {
    pub device_id: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub sessions: u64,
    pub disconnects: u64,
    pub reconnects: u64,
    pub mode_changes: u64,
    pub observed_events: u64,
    pub accumulated_runtime_ms: i64,
    pub connected_since: Option<DateTime<Utc>>,
}

impl DeviceLifetime {
    fn new(event: &ForensicEvent) -> Self {
        Self {
            device_id: event.device_id.clone(),
            first_seen: event.timestamp,
            last_seen: event.timestamp,
            sessions: 0,
            disconnects: 0,
            reconnects: 0,
            mode_changes: 0,
            observed_events: 0,
            accumulated_runtime_ms: 0,
            connected_since: None,
        }
    }

    pub fn total_runtime(&self, now: DateTime<Utc>) -> Duration {
        let active = self
            .connected_since
            .map(|start| now - start)
            .unwrap_or_else(Duration::zero);
        Duration::milliseconds(self.accumulated_runtime_ms) + active
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LifetimeTracker {
    records: BTreeMap<String, DeviceLifetime>,
}

impl LifetimeTracker {
    pub fn observe(&mut self, event: &ForensicEvent) -> &DeviceLifetime {
        let record = self
            .records
            .entry(event.device_id.clone())
            .or_insert_with(|| DeviceLifetime::new(event));
        record.last_seen = event.timestamp;
        record.observed_events = record.observed_events.saturating_add(1);
        match event.kind {
            ForensicEventKind::DeviceConnected => {
                if record.connected_since.is_none() {
                    record.sessions = record.sessions.saturating_add(1);
                    record.connected_since = Some(event.timestamp);
                }
            }
            ForensicEventKind::DeviceReconnected => {
                record.reconnects = record.reconnects.saturating_add(1);
                if record.connected_since.is_none() {
                    record.sessions = record.sessions.saturating_add(1);
                    record.connected_since = Some(event.timestamp);
                }
            }
            ForensicEventKind::DeviceDisconnected => {
                record.disconnects = record.disconnects.saturating_add(1);
                if let Some(start) = record.connected_since.take() {
                    record.accumulated_runtime_ms = record
                        .accumulated_runtime_ms
                        .saturating_add((event.timestamp - start).num_milliseconds().max(0));
                }
            }
            ForensicEventKind::ModeChanged => {
                record.mode_changes = record.mode_changes.saturating_add(1)
            }
            _ => {}
        }
        record
    }

    pub fn get(&self, device_id: &str) -> Option<&DeviceLifetime> {
        self.records.get(device_id)
    }
    pub fn records(&self) -> impl Iterator<Item = &DeviceLifetime> {
        self.records.values()
    }
    pub fn merge_identity(&mut self, previous_id: &str, current_id: &str) {
        if previous_id == current_id {
            return;
        }
        if let Some(mut previous) = self.records.remove(previous_id) {
            previous.device_id = current_id.to_string();
            self.records
                .entry(current_id.to_string())
                .and_modify(|current| {
                    current.first_seen = current.first_seen.min(previous.first_seen);
                    current.last_seen = current.last_seen.max(previous.last_seen);
                    current.sessions = current.sessions.saturating_add(previous.sessions);
                    current.disconnects = current.disconnects.saturating_add(previous.disconnects);
                    current.reconnects = current.reconnects.saturating_add(previous.reconnects);
                    current.mode_changes =
                        current.mode_changes.saturating_add(previous.mode_changes);
                    current.observed_events = current
                        .observed_events
                        .saturating_add(previous.observed_events);
                    current.accumulated_runtime_ms = current
                        .accumulated_runtime_ms
                        .saturating_add(previous.accumulated_runtime_ms);
                })
                .or_insert(previous);
        }
    }
}
