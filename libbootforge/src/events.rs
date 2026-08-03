//! USB device event monitoring.
//!
//! This module preserves the original compatibility monitor and adds the evidence-grade
//! `ForensicEventMonitor`, which consumes normalized scanner output, correlates reconnects,
//! tracks mode and metadata changes, and emits versioned forensic events.

use crate::{
    correlate_reconnect,
    device::DeviceInfo as LegacyDeviceInfo,
    enumeration::enumerate_devices,
    forensic::{ForensicEvent, ForensicEventKind, ObservationSource},
    identity::DeviceIdentity,
    scan_devices,
    types::DeviceInfo,
    Result,
};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

// -----------------------------------------------------------------------------
// Legacy compatibility API
// -----------------------------------------------------------------------------

/// Legacy device event types.
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceEvent {
    Connected(LegacyDeviceInfo),
    Disconnected {
        vendor_id: u16,
        product_id: u16,
        bus_number: u8,
        device_address: u8,
    },
}

/// Legacy bus/address monitor retained for backward compatibility.
pub struct DeviceEventMonitor {
    known_devices: HashMap<(u8, u8), LegacyDeviceInfo>,
}

impl DeviceEventMonitor {
    pub fn new() -> Result<Self> {
        let devices = enumerate_devices()?;
        let known_devices = devices
            .into_iter()
            .map(|device| ((device.bus_number, device.device_address), device))
            .collect();
        Ok(Self { known_devices })
    }

    pub fn poll(&mut self) -> Result<Vec<DeviceEvent>> {
        let current_map: HashMap<(u8, u8), LegacyDeviceInfo> = enumerate_devices()?
            .into_iter()
            .map(|device| ((device.bus_number, device.device_address), device))
            .collect();

        let mut events = Vec::new();

        for (key, device) in &current_map {
            if !self.known_devices.contains_key(key) {
                events.push(DeviceEvent::Connected(device.clone()));
            }
        }

        for (key, device) in &self.known_devices {
            if !current_map.contains_key(key) {
                events.push(DeviceEvent::Disconnected {
                    vendor_id: device.vendor_id,
                    product_id: device.product_id,
                    bus_number: device.bus_number,
                    device_address: device.device_address,
                });
            }
        }

        self.known_devices = current_map;
        Ok(events)
    }

    pub fn wait_for_events(&mut self, timeout: Duration) -> Result<Vec<DeviceEvent>> {
        let poll_interval = Duration::from_millis(100);
        let start = std::time::Instant::now();

        loop {
            let events = self.poll()?;
            if !events.is_empty() || start.elapsed() >= timeout {
                return Ok(events);
            }
            std::thread::sleep(poll_interval);
        }
    }

    pub fn current_devices(&self) -> Vec<&LegacyDeviceInfo> {
        self.known_devices.values().collect()
    }
}

impl Default for DeviceEventMonitor {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            known_devices: HashMap::new(),
        })
    }
}

// -----------------------------------------------------------------------------
// Evidence-grade watcher
// -----------------------------------------------------------------------------

/// Stateful watcher that emits normalized, sequence-numbered forensic events.
pub struct ForensicEventMonitor {
    source: ObservationSource,
    sequence: u64,
    known_devices: HashMap<String, DeviceInfo>,
    recently_disconnected: Vec<DeviceInfo>,
}

impl ForensicEventMonitor {
    /// Start a passive libusb-backed watcher from the current device snapshot.
    pub fn new() -> Result<Self> {
        Self::with_source(ObservationSource::Libusb)
    }

    /// Start a watcher and explicitly identify the backend producing observations.
    pub fn with_source(source: ObservationSource) -> Result<Self> {
        let current = scan_devices()?;
        Ok(Self::from_snapshot(source, current))
    }

    /// Construct a watcher from a supplied snapshot. Useful for deterministic tests and
    /// platform backends that already performed enumeration.
    pub fn from_snapshot(source: ObservationSource, devices: Vec<DeviceInfo>) -> Self {
        let known_devices = devices
            .into_iter()
            .map(|device| (DeviceIdentity::from_device(&device).stable_id, device))
            .collect();

        Self {
            source,
            sequence: 0,
            known_devices,
            recently_disconnected: Vec::new(),
        }
    }

    /// Poll the normalized scanner and emit connected, disconnected, reconnected,
    /// mode-changed, or general changed events.
    pub fn poll(&mut self) -> Result<Vec<ForensicEvent>> {
        let current = scan_devices()?;
        Ok(self.process_snapshot(current))
    }

    /// Process a supplied snapshot through the same correlation pipeline used by `poll`.
    pub fn process_snapshot(&mut self, current: Vec<DeviceInfo>) -> Vec<ForensicEvent> {
        let current_map: HashMap<String, DeviceInfo> = current
            .into_iter()
            .map(|device| (DeviceIdentity::from_device(&device).stable_id, device))
            .collect();

        let previous_ids: HashSet<String> = self.known_devices.keys().cloned().collect();
        let current_ids: HashSet<String> = current_map.keys().cloned().collect();
        let mut events = Vec::new();

        // Record removals first so same-cycle mode transitions can correlate as reconnects
        // even when a device changes VID/PID or identity material between modes.
        for removed_id in previous_ids.difference(&current_ids) {
            if let Some(previous) = self.known_devices.get(removed_id) {
                events.push(self.event(
                    ForensicEventKind::DeviceDisconnected,
                    previous,
                    Some("device no longer present in enumeration snapshot".into()),
                ));
                self.recently_disconnected.push(previous.clone());
            }
        }

        for (stable_id, device) in &current_map {
            if let Some(previous) = self.known_devices.get(stable_id) {
                if previous.mode != device.mode {
                    events.push(self.event(
                        ForensicEventKind::ModeChanged,
                        device,
                        Some(format!("mode changed from {:?} to {:?}", previous.mode, device.mode)),
                    ));
                } else if previous != device {
                    events.push(self.event(
                        ForensicEventKind::DeviceChanged,
                        device,
                        Some("observable device metadata changed".into()),
                    ));
                }
                continue;
            }

            let reconnect = self
                .recently_disconnected
                .iter()
                .enumerate()
                .map(|(index, previous)| (index, correlate_reconnect(previous, device)))
                .filter(|(_, candidate)| candidate.is_match)
                .max_by_key(|(_, candidate)| candidate.score);

            if let Some((index, matched)) = reconnect {
                let previous = self.recently_disconnected.remove(index);
                events.push(self.event(
                    ForensicEventKind::DeviceReconnected,
                    device,
                    Some(format!(
                        "reconnect correlated with score {} and {:?} confidence; previous bus/address {}:{}",
                        matched.score,
                        matched.confidence,
                        previous.bus_number,
                        previous.address
                    )),
                ));
            } else {
                events.push(self.event(
                    ForensicEventKind::DeviceConnected,
                    device,
                    Some("new device identity observed".into()),
                ));
            }
        }

        self.known_devices = current_map;
        events
    }

    /// Wait until one or more forensic events occur or the timeout expires.
    pub fn wait_for_events(&mut self, timeout: Duration) -> Result<Vec<ForensicEvent>> {
        let poll_interval = Duration::from_millis(100);
        let start = std::time::Instant::now();

        loop {
            let events = self.poll()?;
            if !events.is_empty() || start.elapsed() >= timeout {
                return Ok(events);
            }
            std::thread::sleep(poll_interval);
        }
    }

    /// Current normalized device snapshots keyed internally by stable identity.
    pub fn current_devices(&self) -> Vec<&DeviceInfo> {
        self.known_devices.values().collect()
    }

    /// Last sequence number assigned by this watcher.
    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    fn event(
        &mut self,
        kind: ForensicEventKind,
        device: &DeviceInfo,
        message: Option<String>,
    ) -> ForensicEvent {
        self.sequence = self.sequence.saturating_add(1);
        ForensicEvent::from_device(
            self.sequence,
            kind,
            self.source.clone(),
            device,
            message,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        DeviceFamily, DeviceFingerprint, DeviceMode, DevicePlatform, DeviceTransport,
        FingerprintConfidence, WorkflowRecommendation,
    };

    fn device(serial: &str, address: u8, mode: DeviceMode) -> DeviceInfo {
        DeviceInfo {
            bus_number: 1,
            address,
            vendor_id: 0x18d1,
            product_id: 0x4ee7,
            vendor_name: Some("Google".into()),
            manufacturer: Some("Google".into()),
            product_name: Some("Android Device".into()),
            serial_number: Some(serial.into()),
            platform: DevicePlatform::Android,
            transport: DeviceTransport::Usb3,
            mode,
            fingerprint: DeviceFingerprint {
                family: DeviceFamily::AndroidPhone,
                model_hint: None,
                confidence: FingerprintConfidence::High,
            },
            recommended_workflow: WorkflowRecommendation::AndroidAdbWorkflow,
            matched_profile: Some("android-adb".into()),
        }
    }

    #[test]
    fn emits_disconnect_then_correlated_reconnect() {
        let original = device("SERIAL-1", 2, DeviceMode::Adb);
        let mut monitor =
            ForensicEventMonitor::from_snapshot(ObservationSource::Libusb, vec![original]);

        let removed = monitor.process_snapshot(Vec::new());
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].kind, ForensicEventKind::DeviceDisconnected);

        let reconnected = monitor.process_snapshot(vec![device("SERIAL-1", 9, DeviceMode::Adb)]);
        assert_eq!(reconnected.len(), 1);
        assert_eq!(reconnected[0].kind, ForensicEventKind::DeviceReconnected);
        assert_eq!(reconnected[0].sequence, 2);
    }

    #[test]
    fn emits_mode_change_for_same_stable_identity() {
        let original = device("SERIAL-2", 2, DeviceMode::Adb);
        let mut monitor =
            ForensicEventMonitor::from_snapshot(ObservationSource::Libusb, vec![original]);

        let events =
            monitor.process_snapshot(vec![device("SERIAL-2", 2, DeviceMode::Fastboot)]);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, ForensicEventKind::ModeChanged);
    }

    #[test]
    fn emits_connected_for_unseen_identity() {
        let mut monitor =
            ForensicEventMonitor::from_snapshot(ObservationSource::Libusb, Vec::new());

        let events = monitor.process_snapshot(vec![device("SERIAL-3", 4, DeviceMode::Adb)]);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, ForensicEventKind::DeviceConnected);
        assert_eq!(monitor.sequence(), 1);
    }

    #[test]
    #[ignore]
    fn legacy_event_monitor_creation_requires_usb_subsystem() {
        assert!(DeviceEventMonitor::new().is_ok());
    }

    #[test]
    #[ignore]
    fn forensic_monitor_creation_requires_usb_subsystem() {
        assert!(ForensicEventMonitor::new().is_ok());
    }
}
