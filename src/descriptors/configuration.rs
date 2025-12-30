//! Configuration Descriptor Parsing
//!
//! USB Configuration descriptors describe a specific device configuration,
//! including all interfaces and endpoints available in that configuration.

use super::interface::InterfaceDescriptor;
use super::DESCRIPTOR_TIMEOUT;
use serde::{Deserialize, Serialize};

/// USB Configuration Descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationDescriptor {
    /// Configuration number (1-based)
    pub number: u8,
    /// Configuration string descriptor
    pub description: Option<String>,
    /// Total length of data returned for this configuration
    pub total_length: u16,
    /// Number of interfaces in this configuration
    pub num_interfaces: u8,
    /// Configuration attributes
    pub attributes: ConfigurationAttributes,
    /// Maximum power consumption in milliamps
    pub max_power_ma: u16,
    /// All interfaces in this configuration
    pub interfaces: Vec<InterfaceDescriptor>,
    /// Raw extra bytes (class-specific descriptors)
    pub extra: Vec<u8>,
}

/// Configuration attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationAttributes {
    /// Device is self-powered
    pub self_powered: bool,
    /// Device supports remote wakeup
    pub remote_wakeup: bool,
    /// Reserved bit D7 (must be 1 in USB 1.0)
    pub reserved_d7: bool,
}

impl ConfigurationAttributes {
    /// Parse attributes from a byte
    pub fn from_byte(byte: u8) -> Self {
        Self {
            self_powered: (byte & 0x40) != 0,
            remote_wakeup: (byte & 0x20) != 0,
            reserved_d7: (byte & 0x80) != 0,
        }
    }
}

/// Parse a configuration descriptor from rusb
pub fn parse_configuration(
    config: &rusb::ConfigDescriptor,
    handle: Option<&rusb::DeviceHandle<rusb::Context>>,
) -> ConfigurationDescriptor {
    // Try to get configuration string
    let description = handle.and_then(|h| {
        let langs = h.read_languages(DESCRIPTOR_TIMEOUT).ok()?;
        let lang = langs.first()?;
        h.read_string_descriptor(*lang, config.description_string_index()?, DESCRIPTOR_TIMEOUT)
            .ok()
    });
    
    // Parse all interfaces
    let mut interfaces = Vec::new();
    for interface in config.interfaces() {
        for interface_desc in interface.descriptors() {
            let parsed = super::interface::parse_interface(&interface_desc, handle);
            interfaces.push(parsed);
        }
    }
    
    // Get raw attributes byte - rusb doesn't expose this directly, so we reconstruct it
    let self_powered = config.self_powered();
    let remote_wakeup = config.remote_wakeup();
    let attributes_byte = 0x80 | (if self_powered { 0x40 } else { 0 }) | (if remote_wakeup { 0x20 } else { 0 });
    
    ConfigurationDescriptor {
        number: config.number(),
        description,
        total_length: config.total_length(),
        num_interfaces: config.num_interfaces(),
        attributes: ConfigurationAttributes::from_byte(attributes_byte),
        max_power_ma: config.max_power() * 2, // USB spec: value is in 2mA units
        interfaces,
        extra: config.extra().to_vec(),
    }
}

/// Interface Association Descriptor (IAD)
/// Used for multi-interface functions (e.g., CDC, Audio)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceAssociationDescriptor {
    /// First interface number
    pub first_interface: u8,
    /// Number of contiguous interfaces
    pub interface_count: u8,
    /// Function class code
    pub function_class: u8,
    /// Function subclass code
    pub function_subclass: u8,
    /// Function protocol code
    pub function_protocol: u8,
    /// Function string descriptor
    pub function_string: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_attributes() {
        // Self-powered, remote wakeup, D7 set
        let attrs = ConfigurationAttributes::from_byte(0xE0);
        assert!(attrs.self_powered);
        assert!(attrs.remote_wakeup);
        assert!(attrs.reserved_d7);

        // Bus-powered only
        let attrs2 = ConfigurationAttributes::from_byte(0x80);
        assert!(!attrs2.self_powered);
        assert!(!attrs2.remote_wakeup);
        assert!(attrs2.reserved_d7);
    }
}
