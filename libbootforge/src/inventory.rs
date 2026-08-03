//! Deterministic, read-only in-memory inventory for forensic USB events.

use crate::{DeviceLifetime, ForensicEvent, ForensicEventKind, LifetimeTracker};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InventorySnapshot {
    pub generated_at: DateTime<Utc>,
    pub event_count: usize,
    pub device_ids: Vec<String>,
    pub events: Vec<ForensicEvent>,
    pub lifetimes: Vec<DeviceLifetime>,
}

#[derive(Debug, Default, Clone)]
pub struct EventInventory {
    events: Vec<ForensicEvent>,
    by_device: BTreeMap<String, Vec<usize>>,
    by_kind: BTreeMap<String, Vec<usize>>,
    lifetime: LifetimeTracker,
}

impl EventInventory {
    pub fn record(&mut self, event: ForensicEvent) {
        let index = self.events.len();
        self.by_device
            .entry(event.device_id.clone())
            .or_default()
            .push(index);
        self.by_kind
            .entry(kind_key(event.kind).to_string())
            .or_default()
            .push(index);
        self.lifetime.observe(&event);
        self.events.push(event);
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn events_for_device(&self, device_id: &str) -> Vec<&ForensicEvent> {
        self.by_device
            .get(device_id)
            .into_iter()
            .flatten()
            .filter_map(|index| self.events.get(*index))
            .collect()
    }

    pub fn events_of_kind(&self, kind: ForensicEventKind) -> Vec<&ForensicEvent> {
        self.by_kind
            .get(kind_key(kind))
            .into_iter()
            .flatten()
            .filter_map(|index| self.events.get(*index))
            .collect()
    }

    pub fn lifetime(&self, device_id: &str) -> Option<&DeviceLifetime> {
        self.lifetime.get(device_id)
    }

    pub fn snapshot(&self, generated_at: DateTime<Utc>) -> InventorySnapshot {
        let device_ids = self
            .by_device
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let mut lifetimes = self.lifetime.records().cloned().collect::<Vec<_>>();
        lifetimes.sort_by(|left, right| left.device_id.cmp(&right.device_id));
        InventorySnapshot {
            generated_at,
            event_count: self.events.len(),
            device_ids,
            events: self.events.clone(),
            lifetimes,
        }
    }
}

fn kind_key(kind: ForensicEventKind) -> &'static str {
    match kind {
        ForensicEventKind::DeviceObserved => "device_observed",
        ForensicEventKind::DeviceConnected => "device_connected",
        ForensicEventKind::DeviceDisconnected => "device_disconnected",
        ForensicEventKind::DeviceReconnected => "device_reconnected",
        ForensicEventKind::DeviceChanged => "device_changed",
        ForensicEventKind::ModeChanged => "mode_changed",
        ForensicEventKind::DriverChanged => "driver_changed",
        ForensicEventKind::ProtocolObserved => "protocol_observed",
        ForensicEventKind::HealthChanged => "health_changed",
        ForensicEventKind::EnumerationFailed => "enumeration_failed",
        ForensicEventKind::PermissionDenied => "permission_denied",
    }
}
