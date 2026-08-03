//! Deterministic, read-only exports for forensic inventory snapshots.

use crate::{BootforgeError, InventorySnapshot, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const EXPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportManifest {
    pub schema_version: u32,
    pub format: String,
    pub record_count: usize,
    pub sha256: String,
}

pub fn export_inventory_json(snapshot: &InventorySnapshot) -> Result<(String, ExportManifest)> {
    let payload = serde_json::to_string_pretty(snapshot)
        .map_err(|error| BootforgeError::JsonSerializationFailed(error.to_string()))?;
    let manifest = manifest("json", snapshot.event_count, payload.as_bytes());
    Ok((payload, manifest))
}

pub fn export_inventory_csv(snapshot: &InventorySnapshot) -> (String, ExportManifest) {
    let mut output = String::from("sequence,observed_at,kind,source,device_id,vid,pid,bus,address,message\n");
    for event in &snapshot.events {
        let row = [
            event.sequence.to_string(),
            csv(&event.observed_at.to_rfc3339()),
            csv(&format!("{:?}", event.kind)),
            csv(&format!("{:?}", event.source)),
            csv(&event.device_id),
            format!("{:04x}", event.vid),
            format!("{:04x}", event.pid),
            event.bus.map(|value| value.to_string()).unwrap_or_default(),
            event.address.map(|value| value.to_string()).unwrap_or_default(),
            csv(event.message.as_deref().unwrap_or_default()),
        ]
        .join(",");
        output.push_str(&row);
        output.push('\n');
    }
    let manifest = manifest("csv", snapshot.event_count, output.as_bytes());
    (output, manifest)
}

fn csv(value: &str) -> String {
    let escaped = value.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

fn manifest(format: &str, record_count: usize, bytes: &[u8]) -> ExportManifest {
    ExportManifest {
        schema_version: EXPORT_SCHEMA_VERSION,
        format: format.to_string(),
        record_count,
        sha256: format!("{:x}", Sha256::digest(bytes)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn empty_snapshot_exports_deterministically() {
        let snapshot = InventorySnapshot {
            generated_at: Utc::now(),
            event_count: 0,
            device_ids: Vec::new(),
            events: Vec::new(),
            lifetimes: Vec::new(),
        };
        let (left, left_manifest) = export_inventory_csv(&snapshot);
        let (right, right_manifest) = export_inventory_csv(&snapshot);
        assert_eq!(left, right);
        assert_eq!(left_manifest, right_manifest);
    }
}
