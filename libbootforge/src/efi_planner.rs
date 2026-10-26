//! Pure, read-only EFI triage. This module never opens a disk or issues commands.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartitionScheme {
    Gpt,
    Mbr,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EfiObservation {
    pub target_id: String,
    pub scheme: PartitionScheme,
    pub efi_partition_present: bool,
    pub efi_filesystem_readable: bool,
    pub boot_entry_present: bool,
    pub encrypted_volume_locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriageStatus {
    ManualInspection,
    RecoveryCandidate,
    NoRepairIndicated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EfiPlan {
    pub target_id: String,
    pub status: TriageStatus,
    pub findings: Vec<String>,
    pub recommended_checks: Vec<String>,
    pub write_operations: Vec<String>,
}

/// Interpret caller-supplied observations only. Never claim a disk has been inspected.
pub fn plan_efi_recovery(observation: &EfiObservation) -> EfiPlan {
    let mut plan = EfiPlan {
        target_id: observation.target_id.clone(),
        status: TriageStatus::ManualInspection,
        findings: Vec::new(),
        recommended_checks: vec![
            "Verify target identity and obtain a verified backup before any repair".into(),
        ],
        write_operations: Vec::new(),
    };
    if observation.target_id.trim().is_empty() {
        plan.findings.push("Target identity is missing".into());
        return plan;
    }
    if observation.scheme != PartitionScheme::Gpt {
        plan.findings
            .push("GPT EFI recovery cannot be inferred from this partition scheme".into());
        return plan;
    }
    if observation.encrypted_volume_locked {
        plan.findings.push(
            "Encrypted volume is locked; authorized recovery credentials are required".into(),
        );
        return plan;
    }
    if !observation.efi_partition_present {
        plan.findings
            .push("EFI System Partition was not observed".into());
        plan.recommended_checks
            .push("Inspect GPT integrity and available recovery media without writing".into());
        return plan;
    }
    if !observation.efi_filesystem_readable {
        plan.findings.push("EFI filesystem is not readable".into());
        plan.recommended_checks
            .push("Image the affected disk and inspect filesystem damage offline".into());
        return plan;
    }
    if !observation.boot_entry_present {
        plan.status = TriageStatus::RecoveryCandidate;
        plan.findings
            .push("EFI filesystem is readable, but no firmware boot entry was observed".into());
        plan.recommended_checks
            .push("Inspect vendor bootloader paths and firmware boot order".into());
        return plan;
    }
    plan.status = TriageStatus::NoRepairIndicated;
    plan.findings
        .push("Observed EFI structures do not indicate a missing boot entry".into());
    plan.recommended_checks
        .push("Investigate bootloader files, OS integrity, and hardware health".into());
    plan
}

#[cfg(test)]
mod tests {
    use super::*;
    fn baseline() -> EfiObservation {
        EfiObservation {
            target_id: "fixture-disk".into(),
            scheme: PartitionScheme::Gpt,
            efi_partition_present: true,
            efi_filesystem_readable: true,
            boot_entry_present: true,
            encrypted_volume_locked: false,
        }
    }
    #[test]
    fn healthy_is_not_repaired() {
        let p = plan_efi_recovery(&baseline());
        assert_eq!(p.status, TriageStatus::NoRepairIndicated);
        assert!(p.write_operations.is_empty());
    }
    #[test]
    fn missing_boot_entry_is_candidate_only() {
        let mut o = baseline();
        o.boot_entry_present = false;
        let p = plan_efi_recovery(&o);
        assert_eq!(p.status, TriageStatus::RecoveryCandidate);
        assert!(p.write_operations.is_empty());
    }
    #[test]
    fn missing_esp_requires_inspection() {
        let mut o = baseline();
        o.efi_partition_present = false;
        assert_eq!(plan_efi_recovery(&o).status, TriageStatus::ManualInspection);
    }
    #[test]
    fn locked_volume_is_not_repaired() {
        let mut o = baseline();
        o.encrypted_volume_locked = true;
        assert_eq!(plan_efi_recovery(&o).status, TriageStatus::ManualInspection);
    }
    #[test]
    fn unsupported_scheme_is_not_repaired() {
        let mut o = baseline();
        o.scheme = PartitionScheme::Mbr;
        assert_eq!(plan_efi_recovery(&o).status, TriageStatus::ManualInspection);
    }
    #[test]
    fn unreadable_esp_is_not_repaired() {
        let mut o = baseline();
        o.efi_filesystem_readable = false;
        assert_eq!(plan_efi_recovery(&o).status, TriageStatus::ManualInspection);
    }
    #[test]
    fn missing_target_is_not_repaired() {
        let mut o = baseline();
        o.target_id.clear();
        assert_eq!(plan_efi_recovery(&o).status, TriageStatus::ManualInspection);
    }
}
