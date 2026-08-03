//! Driver-state change detection for forensic watcher integration.
//!
//! This layer compares passive native driver reports over time. Platform notification
//! backends may wake the watcher, while this comparator remains the single source of truth
//! for deciding whether a normalized `DriverChanged` event should be emitted.

use crate::driver::DriverReport;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Fields that changed between two driver observations.
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

/// Explained difference between two normalized driver reports.
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
        if previous.backend != current.backend { fields.push(DriverChangeField::Backend); }
        if previous.state != current.state { fields.push(DriverChangeField::State); }
        if previous.confidence != current.confidence { fields.push(DriverChangeField::Confidence); }
        if previous.driver_name != current.driver_name { fields.push(DriverChangeField::DriverName); }
        if previous.service_name != current.service_name { fields.push(DriverChangeField::ServiceName); }
        if previous.provider != current.provider { fields.push(DriverChangeField::Provider); }
        if previous.version != current.version { fields.push(DriverChangeField::Version); }
        if previous.signed != current.signed { fields.push(DriverChangeField::SignatureState); }
        if previous.problem_code != current.problem_code { fields.push(DriverChangeField::ProblemCode); }
        if previous.device_node != current.device_node { fields.push(DriverChangeField::DeviceNode); }
        if previous.evidence != current.evidence { fields.push(DriverChangeField::Evidence); }

        Self {
            changed: !fields.is_empty(),
            fields,
            previous: previous.clone(),
            current: current.clone(),
        }
    }
}

/// Stateful cache of the last native driver report for each stable identity.
#[derive(Debug, Default)]
pub struct DriverStateTracker {
    reports: HashMap<String, DriverReport>,
}

impl DriverStateTracker {
    pub fn seed(&mut self, device_id: impl Into<String>, report: DriverReport) {
        self.reports.insert(device_id.into(), report);
    }

    /// Record a report and return an explained change when one occurred.
    pub fn observe(&mut self, device_id: impl Into<String>, report: DriverReport) -> Option<DriverChange> {
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
        if previous_id == current_id { return; }
        if let Some(report) = self.reports.remove(previous_id) {
            self.reports.entry(current_id.to_string()).or_insert(report);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::{DriverBackend, DriverConfidence, DriverEvidence, DriverState};

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
}
