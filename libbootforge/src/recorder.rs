//! Tamper-evident append-only forensic session recording.

use crate::{BootforgeError, ForensicEvent, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub const RECORD_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceEnvelope {
    pub schema_version: u16,
    pub recorded_at: DateTime<Utc>,
    pub previous_hash: Option<String>,
    pub event: ForensicEvent,
    pub current_hash: String,
}

impl EvidenceEnvelope {
    fn unsigned_payload(
        recorded_at: DateTime<Utc>,
        previous_hash: Option<&str>,
        event: &ForensicEvent,
    ) -> Result<String> {
        serde_json::to_string(&(RECORD_SCHEMA_VERSION, recorded_at, previous_hash, event))
            .map_err(|error| BootforgeError::JsonSerializationFailed(error.to_string()))
    }

    pub fn create(previous_hash: Option<String>, event: ForensicEvent) -> Result<Self> {
        let recorded_at = Utc::now();
        let payload = Self::unsigned_payload(recorded_at, previous_hash.as_deref(), &event)?;
        Ok(Self {
            schema_version: RECORD_SCHEMA_VERSION,
            recorded_at,
            previous_hash,
            event,
            current_hash: sha256_hex(payload.as_bytes()),
        })
    }

    pub fn verify(&self, expected_previous_hash: Option<&str>) -> Result<bool> {
        if self.schema_version != RECORD_SCHEMA_VERSION
            || self.previous_hash.as_deref() != expected_previous_hash
        {
            return Ok(false);
        }
        let payload =
            Self::unsigned_payload(self.recorded_at, self.previous_hash.as_deref(), &self.event)?;
        Ok(self.current_hash == sha256_hex(payload.as_bytes()))
    }

    pub fn to_json_line(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|error| BootforgeError::JsonSerializationFailed(error.to_string()))
    }
}

#[derive(Debug)]
pub struct SessionRecorder {
    path: PathBuf,
    file: File,
    last_hash: Option<String>,
    records_written: u64,
}

impl SessionRecorder {
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)?;
        Ok(Self {
            path,
            file,
            last_hash: None,
            records_written: 0,
        })
    }

    pub fn resume(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let verification = verify_session(&path)?;
        if !verification.valid {
            return Err(BootforgeError::EvidenceChainInvalid(format!(
                "refusing to resume invalid evidence chain at record {:?}",
                verification.first_invalid_record
            )));
        }
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self {
            path,
            file,
            last_hash: verification.last_hash,
            records_written: verification.records,
        })
    }

    pub fn append(&mut self, event: ForensicEvent) -> Result<EvidenceEnvelope> {
        let envelope = EvidenceEnvelope::create(self.last_hash.clone(), event)?;
        writeln!(self.file, "{}", envelope.to_json_line()?)?;
        self.file.flush()?;
        self.last_hash = Some(envelope.current_hash.clone());
        self.records_written = self.records_written.saturating_add(1);
        Ok(envelope)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn records_written(&self) -> u64 {
        self.records_written
    }

    pub fn last_hash(&self) -> Option<&str> {
        self.last_hash.as_deref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationReport {
    pub valid: bool,
    pub records: u64,
    pub first_invalid_record: Option<u64>,
    pub last_hash: Option<String>,
}

pub fn verify_session(path: impl AsRef<Path>) -> Result<VerificationReport> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(VerificationReport {
            valid: true,
            records: 0,
            first_invalid_record: None,
            last_hash: None,
        });
    }

    let file = File::open(path)?;
    let mut previous_hash: Option<String> = None;
    let mut records = 0_u64;

    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        records = records.saturating_add(1);
        let envelope: EvidenceEnvelope = serde_json::from_str(&line)
            .map_err(|error| BootforgeError::JsonSerializationFailed(error.to_string()))?;
        if !envelope.verify(previous_hash.as_deref())? {
            return Ok(VerificationReport {
                valid: false,
                records,
                first_invalid_record: Some(records),
                last_hash: previous_hash,
            });
        }
        previous_hash = Some(envelope.current_hash);
    }

    Ok(VerificationReport {
        valid: true,
        records,
        first_invalid_record: None,
        last_hash: previous_hash,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DeviceFamily, DeviceFingerprint, DeviceInfo, DeviceMode, DevicePlatform, DeviceTransport,
        FingerprintConfidence, ForensicEventKind, ObservationSource, WorkflowRecommendation,
    };
    use std::fs;

    fn event(sequence: u64) -> ForensicEvent {
        let device = DeviceInfo {
            bus_number: 1,
            address: 2,
            vendor_id: 0x18d1,
            product_id: 0x4ee1,
            vendor_name: Some("Google".into()),
            manufacturer: Some("Google".into()),
            product_name: Some("Android ADB".into()),
            serial_number: Some("RECORDER-TEST".into()),
            platform: DevicePlatform::Android,
            transport: DeviceTransport::Usb2,
            mode: DeviceMode::Adb,
            fingerprint: DeviceFingerprint {
                family: DeviceFamily::AndroidPhone,
                model_hint: None,
                confidence: FingerprintConfidence::High,
            },
            recommended_workflow: WorkflowRecommendation::AndroidAdbWorkflow,
            matched_profile: Some("android-adb".into()),
        };
        ForensicEvent::from_device(
            sequence,
            ForensicEventKind::DeviceObserved,
            ObservationSource::Libusb,
            &device,
            None,
        )
    }

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bootforge-{name}-{}-{}.jsonl",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ))
    }

    #[test]
    fn valid_chain_verifies_and_resumes() {
        let path = temp_path("valid-chain");
        {
            let mut recorder = SessionRecorder::create(&path).expect("create recorder");
            recorder.append(event(1)).expect("append first event");
            recorder.append(event(2)).expect("append second event");
        }

        let report = verify_session(&path).expect("verify chain");
        assert!(report.valid);
        assert_eq!(report.records, 2);

        let recorder = SessionRecorder::resume(&path).expect("resume valid chain");
        assert_eq!(recorder.records_written(), 2);
        assert_eq!(recorder.last_hash(), report.last_hash.as_deref());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn tampered_record_is_rejected() {
        let path = temp_path("tampered-chain");
        {
            let mut recorder = SessionRecorder::create(&path).expect("create recorder");
            recorder.append(event(1)).expect("append event");
        }

        let contents = fs::read_to_string(&path).expect("read evidence");
        fs::write(&path, contents.replace("DeviceObserved", "DeviceConnected"))
            .expect("tamper evidence");

        let report = verify_session(&path).expect("verify tampered chain");
        assert!(!report.valid);
        assert_eq!(report.first_invalid_record, Some(1));
        assert!(matches!(
            SessionRecorder::resume(&path),
            Err(BootforgeError::EvidenceChainInvalid(_))
        ));
        let _ = fs::remove_file(path);
    }
}
