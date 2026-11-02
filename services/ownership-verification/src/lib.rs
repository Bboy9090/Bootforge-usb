// ForgeWorks Core - Ownership Verification Service
// COMPLIANCE-FIRST: Verification only, no bypass

use chrono::Utc;
use device_analysis::DeviceProfile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OwnershipAttestation {
    pub user_id: String,
    pub device_id: String,
    pub attestation_type: AttestationType,
    pub documentation_references: Vec<String>, // Hashed/document references only
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttestationType {
    PurchaseReceipt,
    CourtOrder,
    InheritanceDocument,
    GiftDocument,
    EnterpriseAuthorization,
    ServiceCenterAuthorization,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerificationResult {
    pub verified: bool,
    pub confidence: f64, // 0.0 - 1.0
    pub required_authorization: Option<AuthorizationRequirement>,
    pub blocked: bool,
    pub verification_timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthorizationRequirement {
    OwnershipProof,
    CourtOrder,
    OEMAuthorization,
    CarrierAuthorization,
    ServiceCenterAuthorization,
}

/**
 * Verify ownership claim
 *
 * This function verifies ownership through attestation.
 * It does NOT bypass any locks or security features.
 */
pub fn verify_ownership(
    attestation: &OwnershipAttestation,
    device: &DeviceProfile,
) -> VerificationResult {
    let confidence = calculate_confidence(attestation, device);
    let required_auth = determine_authorization_requirement(&confidence, device);
    let verified = confidence >= 0.85; // Threshold for verification
    let blocked = confidence < 0.50; // Too low = block

    VerificationResult {
        verified,
        confidence,
        required_authorization: required_auth,
        blocked,
        verification_timestamp: Utc::now(),
    }
}

fn calculate_confidence(attestation: &OwnershipAttestation, device: &DeviceProfile) -> f64 {
    // Base confidence from attestation type
    let base_confidence: f64 = match attestation.attestation_type {
        AttestationType::CourtOrder => 0.95,
        AttestationType::ServiceCenterAuthorization => 0.90,
        AttestationType::EnterpriseAuthorization => 0.85,
        AttestationType::PurchaseReceipt => 0.80,
        AttestationType::InheritanceDocument => 0.75,
        AttestationType::GiftDocument => 0.70,
    };

    // Adjust based on documentation quality
    let doc_quality: f64 = if attestation.documentation_references.is_empty() {
        0.5
    } else if attestation.documentation_references.len() >= 3 {
        1.0
    } else {
        0.8
    };

    // Adjust based on device classification (high-risk devices require higher confidence)
    let device_adjustment: f64 = match device.classification {
        device_analysis::DeviceClassification::ServiceModified => 0.7,
        device_analysis::DeviceClassification::HardwareModified => 0.8,
        device_analysis::DeviceClassification::SoftwareModified => 0.9,
        device_analysis::DeviceClassification::Clean => 1.0,
        device_analysis::DeviceClassification::Unknown => 0.8,
    };

    let result: f64 = base_confidence * doc_quality * device_adjustment;
    if result > 1.0 {
        1.0
    } else {
        result
    }
}

fn determine_authorization_requirement(
    confidence: &f64,
    device: &DeviceProfile,
) -> Option<AuthorizationRequirement> {
    if *confidence < 0.85 {
        // High-risk devices require specific authorization
        match device.classification {
            device_analysis::DeviceClassification::ServiceModified => {
                Some(AuthorizationRequirement::ServiceCenterAuthorization)
            }
            device_analysis::DeviceClassification::HardwareModified => {
                Some(AuthorizationRequirement::OEMAuthorization)
            }
            _ => Some(AuthorizationRequirement::OwnershipProof),
        }
    } else {
        None
    }
}

/**
 * Gate check: Ownership verification required before proceeding
 */
pub fn require_ownership_verification(device: &DeviceProfile) -> bool {
    // Always require for high-risk scenarios
    matches!(
        device.classification,
        device_analysis::DeviceClassification::ServiceModified
            | device_analysis::DeviceClassification::HardwareModified
            | device_analysis::DeviceClassification::Unknown
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unknown_profile() -> DeviceProfile {
        device_analysis::analyze("untrusted metadata")
    }

    #[test]
    fn unknown_state_requires_verification_and_cannot_gain_high_confidence_from_metadata() {
        let device = unknown_profile();
        let attestation = OwnershipAttestation {
            user_id: "user123".to_string(),
            device_id: device.device_id.clone(),
            attestation_type: AttestationType::CourtOrder,
            documentation_references: vec![
                "doc1".to_string(),
                "doc2".to_string(),
                "doc3".to_string(),
            ],
            timestamp: Utc::now(),
        };

        let result = verify_ownership(&attestation, &device);
        assert!(result.confidence < 0.85);
        assert!(!result.verified);
        assert_eq!(
            result.required_authorization,
            Some(AuthorizationRequirement::OwnershipProof)
        );
        assert!(require_ownership_verification(&device));
    }
}
