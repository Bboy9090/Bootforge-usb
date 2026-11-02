// ForgeWorks Core - Legal Classification Service
// COMPLIANCE-FIRST: Classification and routing only, no execution

use device_analysis::DeviceProfile;
use ownership_verification::VerificationResult;
use serde::{Deserialize, Serialize};

pub mod loader;
pub use loader::{
    get_authorization_requirements, get_default_status, load_all_jurisdictions, load_jurisdiction,
    JurisdictionMap, LoaderError,
};

// Re-export RiskLevel for use in other services
pub use device_analysis::RiskLevel;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LegalStatus {
    Permitted,
    ConditionallyPermitted,
    Prohibited,
    RequiresAuthorization,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Jurisdiction {
    US,
    EU,
    UK,
    Canada,
    Australia,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LegalClassification {
    pub status: LegalStatus,
    pub jurisdiction: Jurisdiction,
    pub authorization_required: Vec<AuthorizationType>,
    pub risk_level: RiskLevel,
    pub routing_instructions: RoutingInstructions,
    pub precedent_references: Vec<String>, // Abstract references only
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthorizationType {
    OwnershipProof,
    CourtOrder,
    OEMAuthorization,
    CarrierAuthorization,
    ServiceCenterAuthorization,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoutingInstructions {
    pub route_to: RouteTarget,
    pub contact_information: String, // Generic contact info
    pub required_documentation: Vec<String>,
    pub compliance_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RouteTarget {
    OEM,
    Carrier,
    ServiceCenter,
    CourtSystem,
    LegalCounsel,
}

/**
 * Classify legal status based on device scenario
 *
 * This function classifies legal status for routing purposes.
 * It does NOT provide legal advice or execution instructions.
 */
pub fn classify_legal_status(
    device: &DeviceProfile,
    ownership: &VerificationResult,
    jurisdiction: Jurisdiction,
) -> LegalClassification {
    let (status, auth_required, risk) = determine_legal_status(device, ownership, &jurisdiction);
    let routing = generate_routing_instructions(&status, &jurisdiction, &auth_required);

    LegalClassification {
        status: status.clone(),
        jurisdiction, // Copy trait allows reuse
        authorization_required: auth_required.clone(),
        risk_level: risk.clone(),
        routing_instructions: routing,
        precedent_references: vec![
            "DMCA Section 1201".to_string(),
            "CFAA".to_string(),
            "GDPR (EU only)".to_string(),
        ],
    }
}

fn determine_legal_status(
    device: &DeviceProfile,
    ownership: &VerificationResult,
    _jurisdiction: &Jurisdiction,
) -> (LegalStatus, Vec<AuthorizationType>, RiskLevel) {
    match device.classification {
        device_analysis::DeviceClassification::ServiceModified => (
            LegalStatus::Prohibited,
            vec![
                AuthorizationType::CourtOrder,
                AuthorizationType::ServiceCenterAuthorization,
            ],
            RiskLevel::VeryHigh,
        ),
        device_analysis::DeviceClassification::HardwareModified => (
            LegalStatus::RequiresAuthorization,
            vec![
                AuthorizationType::OEMAuthorization,
                AuthorizationType::OwnershipProof,
            ],
            RiskLevel::High,
        ),
        device_analysis::DeviceClassification::SoftwareModified => {
            if ownership.verified {
                (
                    LegalStatus::ConditionallyPermitted,
                    vec![AuthorizationType::OwnershipProof],
                    RiskLevel::Medium,
                )
            } else {
                (
                    LegalStatus::RequiresAuthorization,
                    vec![AuthorizationType::OwnershipProof],
                    RiskLevel::Medium,
                )
            }
        }
        device_analysis::DeviceClassification::Clean => {
            (LegalStatus::Permitted, vec![], RiskLevel::Low)
        }
        device_analysis::DeviceClassification::Unknown => (
            LegalStatus::RequiresAuthorization,
            vec![AuthorizationType::OwnershipProof],
            RiskLevel::Medium,
        ),
    }
}

fn generate_routing_instructions(
    status: &LegalStatus,
    _jurisdiction: &Jurisdiction,
    auth_required: &[AuthorizationType],
) -> RoutingInstructions {
    match status {
        LegalStatus::Prohibited | LegalStatus::RequiresAuthorization => {
            let route_target = if auth_required.contains(&AuthorizationType::CourtOrder) {
                RouteTarget::CourtSystem
            } else if auth_required.contains(&AuthorizationType::OEMAuthorization) {
                RouteTarget::OEM
            } else if auth_required.contains(&AuthorizationType::ServiceCenterAuthorization) {
                RouteTarget::ServiceCenter
            } else {
                RouteTarget::LegalCounsel
            };

            RoutingInstructions {
                route_to: route_target,
                contact_information: "Contact legal counsel for jurisdiction-specific guidance"
                    .to_string(),
                required_documentation: vec![
                    "Ownership proof".to_string(),
                    "Authorization documents".to_string(),
                ],
                compliance_notes: "External authority routing required".to_string(),
            }
        }
        _ => RoutingInstructions {
            route_to: RouteTarget::OEM,
            contact_information: "Contact OEM service program".to_string(),
            required_documentation: vec!["Ownership proof".to_string()],
            compliance_notes: "Standard routing".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(classification: device_analysis::DeviceClassification) -> DeviceProfile {
        DeviceProfile {
            device_id: "evidence-test".to_string(),
            model: "Observed model".to_string(),
            manufacturer: "Observed manufacturer".to_string(),
            security_state: "test evidence state".to_string(),
            capability_class: "test".to_string(),
            classification,
            restrictions: vec![],
            analysis_timestamp: chrono::Utc::now(),
            non_invasive: true,
        }
    }

    fn verified_ownership() -> VerificationResult {
        VerificationResult {
            verified: true,
            confidence: 0.9,
            required_authorization: None,
            blocked: false,
            verification_timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn explicit_service_modified_evidence_routes_as_prohibited() {
        let result = classify_legal_status(
            &profile(device_analysis::DeviceClassification::ServiceModified),
            &verified_ownership(),
            Jurisdiction::US,
        );
        assert_eq!(result.status, LegalStatus::Prohibited);
        assert_eq!(result.risk_level, RiskLevel::VeryHigh);
    }

    #[test]
    fn explicit_clean_evidence_can_route_as_permitted() {
        let result = classify_legal_status(
            &profile(device_analysis::DeviceClassification::Clean),
            &verified_ownership(),
            Jurisdiction::US,
        );
        assert_eq!(result.status, LegalStatus::Permitted);
        assert_eq!(result.risk_level, RiskLevel::Low);
    }

    #[test]
    fn unknown_device_state_fails_closed_to_authorization() {
        let result = classify_legal_status(
            &profile(device_analysis::DeviceClassification::Unknown),
            &verified_ownership(),
            Jurisdiction::US,
        );
        assert_eq!(result.status, LegalStatus::RequiresAuthorization);
        assert_eq!(result.risk_level, RiskLevel::Medium);
        assert!(result
            .authorization_required
            .contains(&AuthorizationType::OwnershipProof));
    }
}
