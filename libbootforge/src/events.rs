//! USB device event monitoring.
//!
//! Preserves the original compatibility monitor and provides an evidence-grade watcher
//! that emits stable, normalized forensic events with stateful health intelligence.

use crate::{
    correlate_reconnect,
    device::DeviceInfo as LegacyDeviceInfo,
    enumeration::enumerate_devices,
    forensic::{ForensicEvent, ForensicEventKind, ObservationSource},
    health::{HealthReport, HealthTracker},
    identity::DeviceIdentity,
    scan_devices,
    types::DeviceInfo,
    Result,
};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

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

/// Legacy bus/address watcher retained for API compatibility.
pub struct DeviceEventMonitor {
    known_devices: HashMap<(u8, u8), LegacyDeviceInfo>,
}

impl DeviceEventMonitor {
    pub fn new() -> Result<Self> {
        let known_devices = enumerate_devices()?
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
        let start = std::time::Instant::now();
        loop {
            let events = self.poll()?;
            if !events.is_empty() || start.elapsed() >= timeout {
                return Ok(events);
            }
            std::thread::sleep(Duration::from_millis(100));
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

/// Evidence-grade watcher with stable identity, reconnect correlation, ordered events,
/// passive protocol classification, and stateful connection-health tracking.
pub struct ForensicEventMonitor {
    source: ObservationSource,
    sequence: u64,
    known_devices: HashMap<String, DeviceInfo>,
    recently_disconnected: Vec<(String, DeviceInfo)>,
    health_tracker: HealthTracker,
}

impl ForensicEventMonitor {
    pub fn new() -> Result<Self> {
        Self::with_source(ObservationSource::Libusb)
    }

    pub fn with_source(source: ObservationSource) -> Result<Self> {
        Ok(Self::from_snapshot(source, scan_devices()?))
    }

    pub fn from_snapshot(source: ObservationSource, devices: Vec<DeviceInfo>) -> Self {
        let known_devices: HashMap<String, DeviceInfo> = devices
            .into_iter()
            .map(|device| (DeviceIdentity::from_device(&device).stable_id, device))
            .collect();
        let mut health_tracker = HealthTracker::default();
        for stable_id in known_devices.keys() {
            health_tracker.record_enumeration_success(stable_id);
        }

        Self {
            source,
            sequence: 0,
            known_devices,
            recently_disconnected: Vec::new(),
            health_tracker,
        }
    }

    pub fn poll(&mut self) -> Result<Vec<ForensicEvent>> {
        Ok(self.process_snapshot(scan_devices()?))
    }

    pub fn process_snapshot(&mut self, current: Vec<DeviceInfo>) -> Vec<ForensicEvent> {
        let current_map: HashMap<String, DeviceInfo> = current
            .into_iter()
            .map(|device| (DeviceIdentity::from_device(&device).stable_id, device))
            .collect();

        let previous_ids: HashSet<String> = self.known_devices.keys().cloned().collect();
        let current_ids: HashSet<String> = current_map.keys().cloned().collect();
        let mut events = Vec::new();

        for removed_id in previous_ids.difference(&current_ids) {
            if let Some(previous) = self.known_devices.get(removed_id).cloned() {
                let report = self.health_tracker.record_disconnect(removed_id, Utc::now());
                events.push(self.event_with_health(
                    ForensicEventKind::DeviceDisconnected,
                    &previous,
                    Some("device no longer present in enumeration snapshot".into()),
                    report,
                ));
                self.recently_disconnected
                    .push((removed_id.clone(), previous));
            }
        }

        for (stable_id, device) in &current_map {
            if let Some(previous) = self.known_devices.get(stable_id).cloned() {
                if previous.mode != device.mode {
                    self.health_tracker
                        .record_enumeration_success(stable_id);
                    let report = self.health_tracker.record_mode_transition(stable_id);
                    let message =
                        format!("mode changed from {:?} to {:?}", previous.mode, device.mode);
                    events.push(self.event_with_health(
                        ForensicEventKind::ModeChanged,
                        device,
                        Some(message),
                        report,
                    ));
                } else if previous != *device {
                    let report = self
                        .health_tracker
                        .record_enumeration_success(stable_id);
                    events.push(self.event_with_health(
                        ForensicEventKind::DeviceChanged,
                        device,
                        Some("observable device metadata changed".into()),
                        report,
                    ));
                }
                continue;
            }

            let reconnect = self
                .recently_disconnected
                .iter()
                .enumerate()
                .map(|(index, (_, previous))| (index, correlate_reconnect(previous, device)))
                .filter(|(_, candidate)| candidate.is_match)
                .max_by_key(|(_, candidate)| candidate.score);

            if let Some((index, matched)) = reconnect {
                let (previous_id, previous) = self.recently_disconnected.remove(index);
                self.health_tracker
                    .merge_identity(&previous_id, stable_id);
                self.health_tracker
                    .record_enumeration_success(stable_id);
                let report = self.health_tracker.record_reconnect(stable_id, Utc::now());
                let message = format!(
                    "reconnect correlated with score {} and {:?} confidence; previous bus/address {}:{}",
                    matched.score,
                    matched.confidence,
                    previous.bus_number,
                    previous.address
                );
                events.push(self.event_with_health(
                    ForensicEventKind::DeviceReconnected,
                    device,
                    Some(message),
                    report,
                ));
            } else {
                let report = self
                    .health_tracker
                    .record_enumeration_success(stable_id);
                events.push(self.event_with_health(
                    ForensicEventKind::DeviceConnected,
                    device,
                    Some("new device identity observed".into()),
                    report,
                ));
            }
        }

        self.known_devices = current_map;
        events
    }

    pub fn wait_for_events(&mut self, timeout: Duration) -> Result<Vec<ForensicEvent>> {
        let start = std::time::Instant::now();
        loop {
            let events = self.poll()?;
            if !events.is_empty() || start.elapsed() >= timeout {
                return Ok(events);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn current_devices(&self) -> Vec<&DeviceInfo> {
        self.known_devices.values().collect()
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Return the current health report for a stable device identity.
    pub fn health_report(&self, device_id: &str) -> HealthReport {
        self.health_tracker.report(device_id)
    }

    fn event_with_health(
        &mut self,
        kind: ForensicEventKind,
        device: &DeviceInfo,
        message: Option<String>,
        report: HealthReport,
    ) -> ForensicEvent {
        self.sequence = self.sequence.saturating_add(1);
        ForensicEvent::from_device(
            self.sequence,
            kind,
            self.source.clone(),
            device,
            message,
        )
        .with_health_report(report)
    }
}

impl Default for ForensicEventMonitor {
    fn default() -> Self {
        Self::from_snapshot(ObservationSource::Libusb, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::{HealthSignal, HealthState};
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
    fn disconnect_and_reconnect_carry_accumulated_health() {
        let original = device("SERIAL-1", 2, DeviceMode::Adb);
        let stable_id = DeviceIdentity::from_device(&original).stable_id;
        let mut monitor = ForensicEventMonitor::from_snapshot(
            ObservationSource::Libusb,
            vec![original],
        );

        let removed = monitor.process_snapshot(Vec::new());
        assert_eq!(removed[0].kind, ForensicEventKind::DeviceDisconnected);
        assert_eq!(removed[0].health_report.disconnect_count, 1);

        let reconnected = monitor.process_snapshot(vec![device(
            "SERIAL-1",
            9,
            DeviceMode::Adb,
        )]);
        assert_eq!(reconnected[0].kind, ForensicEventKind::DeviceReconnected);
        assert_eq!(reconnected[0].health_report.reconnect_count, 1);
        assert_eq!(reconnected[0].health_report.rapid_reconnect_count, 1);
        assert!(reconnected[0]
            .health_report
            .signals
            .contains(&HealthSignal::RapidReconnect));
        assert_ne!(monitor.health_report(&stable_id).state, HealthState::Unknown);
    }

    #[test]
    fn mode_change_updates_health_history() {
        let original = device("SERIAL-2", 2, DeviceMode::Adb);
        let mut monitor = ForensicEventMonitor::from_snapshot(
            ObservationSource::Libusb,
            vec![original],
        );

        let events =
            monitor.process_snapshot(vec![device("SERIAL-2", 2, DeviceMode::Fastboot)]);
        assert_eq!(events[0].kind, ForensicEventKind::ModeChanged);
        assert_eq!(events[0].health_report.mode_transition_count, 1);
        assert!(events[0]
            .health_report
            .signals
            .contains(&HealthSignal::ModeTransition));
    }

    #[test]
    fn unseen_device_gets_evidence_backed_healthy_report() {
        let mut monitor =
            ForensicEventMonitor::from_snapshot(ObservationSource::Libusb, Vec::new());

        let events =
            monitor.process_snapshot(vec![device("SERIAL-3", 4, DeviceMode::Adb)]);
        assert_eq!(events[0].kind, ForensicEventKind::DeviceConnected);
        assert_eq!(events[0].health_report.state, HealthState::Healthy);
        assert_eq!(events[0].health_report.enumeration_success_count, 1);
    }

    #[test]
    #[ignore]
    fn forensic_monitor_creation_requires_usb_subsystem() {
        assert!(ForensicEventMonitor::new().is_ok());
    }
}
