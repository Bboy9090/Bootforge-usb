//! Driver-state change detection for forensic watcher integration.
//!
//! Native backends may wake this monitor, but normalized report comparison remains the single
//! source of truth for deciding whether a `DriverChanged` event occurred.

use crate::driver::DriverReport;
use crate::forensic::{ForensicEvent, ForensicEventKind, ObservationSource};
use crate::identity::DeviceIdentity;
use crate::notification::{NotificationWake, PollingWake, WakeResult};
use crate::{inspect_platform_driver, scan_devices, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DriverChangeField {
    Backend,
    State,
    Confidence,
    DriverName,
    ServiceName,
    Provider,
    Version,
    SignatureState,
    ProblemCode,
    DeviceNode,
    Evidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DriverChange {
    pub changed: bool,
    pub fields: Vec<DriverChangeField>,
    pub previous: DriverReport,
    pub current: DriverReport,
}

impl DriverChange {
    pub fn between(previous: &DriverReport, current: &DriverReport) -> Self {
        let mut fields = Vec::new();
        if previous.backend != current.backend {
            fields.push(DriverChangeField::Backend);
        }
        if previous.state != current.state {
            fields.push(DriverChangeField::State);
        }
        if previous.confidence != current.confidence {
            fields.push(DriverChangeField::Confidence);
        }
        if previous.driver_name != current.driver_name {
            fields.push(DriverChangeField::DriverName);
        }
        if previous.service_name != current.service_name {
            fields.push(DriverChangeField::ServiceName);
        }
        if previous.provider != current.provider {
            fields.push(DriverChangeField::Provider);
        }
        if previous.version != current.version {
            fields.push(DriverChangeField::Version);
        }
        if previous.signed != current.signed {
            fields.push(DriverChangeField::SignatureState);
        }
        if previous.problem_code != current.problem_code {
            fields.push(DriverChangeField::ProblemCode);
        }
        if previous.device_node != current.device_node {
            fields.push(DriverChangeField::DeviceNode);
        }
        if previous.evidence != current.evidence {
            fields.push(DriverChangeField::Evidence);
        }
        Self {
            changed: !fields.is_empty(),
            fields,
            previous: previous.clone(),
            current: current.clone(),
        }
    }
}

#[derive(Debug, Default)]
pub struct DriverStateTracker {
    reports: HashMap<String, DriverReport>,
}

impl DriverStateTracker {
    pub fn seed(&mut self, device_id: impl Into<String>, report: DriverReport) {
        self.reports.insert(device_id.into(), report);
    }

    pub fn observe(
        &mut self,
        device_id: impl Into<String>,
        report: DriverReport,
    ) -> Option<DriverChange> {
        let device_id = device_id.into();
        let change = self
            .reports
            .get(&device_id)
            .map(|previous| DriverChange::between(previous, &report))
            .filter(|change| change.changed);
        self.reports.insert(device_id, report);
        change
    }

    pub fn report(&self, device_id: &str) -> Option<&DriverReport> {
        self.reports.get(device_id)
    }

    pub fn remove(&mut self, device_id: &str) -> Option<DriverReport> {
        self.reports.remove(device_id)
    }

    pub fn merge_identity(&mut self, previous_id: &str, current_id: &str) {
        if previous_id == current_id {
            return;
        }
        if let Some(report) = self.reports.remove(previous_id) {
            self.reports.entry(current_id.to_string()).or_insert(report);
        }
    }
}

/// Driver watcher using a pluggable wake source and passive native report comparison.
pub struct DriverChangeMonitor {
    source: ObservationSource,
    sequence: u64,
    tracker: DriverStateTracker,
    wake: Box<dyn NotificationWake>,
    last_wake: Option<WakeResult>,
}

impl DriverChangeMonitor {
    /// Construct with the portable polling wake source.
    pub fn new() -> Result<Self> {
        Self::with_wake(Box::new(PollingWake::new(Duration::from_millis(250))))
    }

    /// Construct with a native or custom wake source.
    pub fn with_wake(wake: Box<dyn NotificationWake>) -> Result<Self> {
        let mut monitor = Self {
            source: native_source(),
            sequence: 0,
            tracker: DriverStateTracker::default(),
            wake,
            last_wake: None,
        };
        for device in scan_devices()? {
            let id = DeviceIdentity::from_device(&device).stable_id;
            monitor.tracker.seed(id, inspect_platform_driver(&device));
        }
        Ok(monitor)
    }

    pub fn poll(&mut self) -> Result<Vec<ForensicEvent>> {
        let mut events = Vec::new();
        for device in scan_devices()? {
            let id = DeviceIdentity::from_device(&device).stable_id;
            let report = inspect_platform_driver(&device);
            if let Some(change) = self.tracker.observe(id, report.clone()) {
                self.sequence = self.sequence.saturating_add(1);
                events.push(
                    ForensicEvent::from_device(
                        self.sequence,
                        ForensicEventKind::DriverChanged,
                        self.source.clone(),
                        &device,
                        Some(format!("driver report changed: {:?}", change.fields)),
                    )
                    .with_driver_report(report),
                );
            }
        }
        Ok(events)
    }

    /// Wait for a native/polling wake, then rescan and compare normalized reports.
    pub fn wait_for_events(&mut self, timeout: Duration) -> Result<Vec<ForensicEvent>> {
        let start = Instant::now();
        loop {
            let remaining = timeout.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                return Ok(Vec::new());
            }
            self.last_wake = Some(self.wake.wait(remaining));
            let events = self.poll()?;
            if !events.is_empty() || start.elapsed() >= timeout {
                return Ok(events);
            }
        }
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn last_wake(&self) -> Option<WakeResult> {
        self.last_wake
    }
}

fn native_source() -> ObservationSource {
    #[cfg(windows)]
    {
        return ObservationSource::WindowsSetupApi;
    }
    #[cfg(target_os = "linux")]
    {
        return ObservationSource::LinuxSysfs;
    }
    #[cfg(target_os = "macos")]
    {
        return ObservationSource::MacOsIoKit;
    }
    #[cfg(feature = "arcwyre")]
    {
        return ObservationSource::ArcwyreNative;
    }
    #[allow(unreachable_code)]
    ObservationSource::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::{DriverBackend, DriverConfidence, DriverEvidence, DriverState};
    use crate::notification::{NotificationSignal, WakeReason};

    fn report(state: DriverState, service: Option<&str>) -> DriverReport {
        DriverReport {
            backend: DriverBackend::WindowsSetupApi,
            state,
            confidence: DriverConfidence::High,
            driver_name: service.map(str::to_string),
            service_name: service.map(str::to_string),
            provider: Some("Microsoft".into()),
            version: None,
            signed: None,
            problem_code: None,
            device_node: Some("USB\\VID_1234&PID_5678".into()),
            evidence: vec![DriverEvidence::BackendRecord],
            message: None,
        }
    }

    #[test]
    fn unchanged_report_emits_nothing() {
        let first = report(DriverState::Bound, Some("WinUSB"));
        let mut tracker = DriverStateTracker::default();
        tracker.seed("device", first.clone());
        assert_eq!(tracker.observe("device", first), None);
    }

    #[test]
    fn state_and_service_changes_are_explained() {
        let mut tracker = DriverStateTracker::default();
        tracker.seed("device", report(DriverState::Present, None));
        let change = tracker
            .observe("device", report(DriverState::Bound, Some("WinUSB")))
            .expect("change expected");
        assert!(change.fields.contains(&DriverChangeField::State));
        assert!(change.fields.contains(&DriverChangeField::ServiceName));
    }

    #[test]
    fn notification_signal_reports_native_wake() {
        let signal = NotificationSignal::default();
        signal.notify();
        let result = signal.wait(Duration::from_millis(1));
        assert_eq!(result.reason, WakeReason::NativeNotification);
    }
}
