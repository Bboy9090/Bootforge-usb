// ForgeWorks Core - Jurisdiction Map Loader
// Loads jurisdiction-specific legal rules from JSON files

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JurisdictionMap {
    pub name: String,
    pub code: String,
    pub default: String,
    pub right_to_repair: String,
    pub notes: Vec<String>,
    pub authorization_requirements: HashMap<String, Vec<String>>,
    pub risk_guidelines: HashMap<String, String>,
}

/**
 * Load jurisdiction map from JSON file
 */
pub fn load_jurisdiction(code: &str) -> Result<JurisdictionMap, String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let file_path = format!(
        "{}/jurisdiction-map/{}.json",
        manifest_dir,
        code.to_lowercase()
    );

    if !Path::new(&file_path).exists() {
        return Err(format!("Jurisdiction map not found: {}", code));
    }

    let contents = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read jurisdiction file: {}", e))?;

    let map: JurisdictionMap = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse jurisdiction JSON: {}", e))?;

    Ok(map)
}

/**
 * Load all available jurisdiction maps
 */
pub fn load_all_jurisdictions() -> HashMap<String, JurisdictionMap> {
    let codes = vec!["us", "eu", "uk", "ca", "au", "global"];
    let mut maps = HashMap::new();

    for code in codes {
        if let Ok(map) = load_jurisdiction(code) {
            maps.insert(code.to_string(), map);
        }
    }

    maps
}

/**
 * Get default status for a jurisdiction
 */
pub fn get_default_status(jurisdiction_code: &str) -> String {
    if let Ok(map) = load_jurisdiction(jurisdiction_code) {
        map.default.clone()
    } else {
        "requires_authorization".to_string()
    }
}

/**
 * Get authorization requirements for a jurisdiction and classification
 */
pub fn get_authorization_requirements(
    jurisdiction_code: &str,
    classification: &str,
) -> Vec<String> {
    if let Ok(map) = load_jurisdiction(jurisdiction_code) {
        map.authorization_requirements
            .get(classification)
            .cloned()
            .unwrap_or_default()
    } else {
        // Fallback to global/default requirements
        vec![
            "ownership_proof".to_string(),
            "local_legal_counsel".to_string(),
        ]
    }
}

#[derive(Debug)]
pub enum LoaderError {
    FileNotFound(String),
    ParseError(String),
    IoError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_jurisdiction() {
        // Test loading a known jurisdiction
        let result = load_jurisdiction("us");
        assert!(result.is_ok());

        if let Ok(map) = result {
            assert_eq!(map.code, "US");
            assert!(!map.notes.is_empty());
        }
    }

    #[test]
    fn test_load_nonexistent_jurisdiction() {
        let result = load_jurisdiction("xx");
        assert!(result.is_err());
    }
}
