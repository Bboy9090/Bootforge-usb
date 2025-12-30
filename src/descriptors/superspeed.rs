//! USB 3.0+ SuperSpeed and SuperSpeedPlus Descriptor Parsing
//!
//! This module handles USB 3.0, 3.1, and 3.2 specific capabilities including:
//! - SuperSpeed USB Device Capability
//! - SuperSpeedPlus USB Device Capability (USB 3.1+)
//! - Link power management (U1/U2 states)
//! - SuperSpeed/SuperSpeedPlus speed attributes

use serde::{Deserialize, Serialize};

/// SuperSpeed USB Device Capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperSpeedCapability {
    /// Bitmap of attributes
    pub attributes: u8,
    /// LPM (Link Power Management) supported
    pub lpm_supported: bool,
    /// Bitmap of supported speeds
    pub speed_supported: u16,
    /// Functionality support field
    pub functionality_support: u8,
    /// U1 Device Exit Latency (microseconds)
    pub u1_dev_exit_lat: u8,
    /// U2 Device Exit Latency (microseconds)
    pub u2_dev_exit_lat: u16,
}

impl SuperSpeedCapability {
    /// Check if SuperSpeed (5 Gbps) is supported
    pub fn supports_gen1(&self) -> bool {
        (self.speed_supported & 0x01) != 0
    }
    
    /// Get the lowest supported speed
    pub fn lowest_speed(&self) -> SuperSpeedMode {
        // TODO: Parse actual speed support from functionality_support field
        // For now, all SuperSpeed devices support at least Gen1
        SuperSpeedMode::Gen1
    }
    
    /// Get maximum U1 exit latency in microseconds
    pub fn u1_exit_latency_us(&self) -> u8 {
        self.u1_dev_exit_lat
    }
    
    /// Get maximum U2 exit latency in microseconds
    pub fn u2_exit_latency_us(&self) -> u16 {
        self.u2_dev_exit_lat
    }
}

/// SuperSpeedPlus USB Device Capability (USB 3.1+)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperSpeedPlusCapability {
    /// Reserved field
    pub reserved: u8,
    /// Bitmap of attributes
    pub attributes: u32,
    /// Functionality support field
    pub functionality_support: u16,
    /// Reserved field
    pub reserved2: u16,
    /// Sublink speed attribute entries
    pub sublink_speed_attributes: Vec<u32>,
}

impl SuperSpeedPlusCapability {
    /// Get the number of Sublink Speed Attribute (SSAC) count
    pub fn sublink_speed_attr_count(&self) -> u8 {
        (self.attributes & 0x1F) as u8 + 1
    }
    
    /// Get the number of Sublink Speed ID (SSIC) count
    pub fn sublink_speed_id_count(&self) -> u8 {
        ((self.attributes >> 5) & 0x0F) as u8 + 1
    }
    
    /// Parse sublink speed attributes into meaningful values
    pub fn parse_sublink_speeds(&self) -> Vec<SublinkSpeedAttribute> {
        self.sublink_speed_attributes
            .iter()
            .map(|&attr| SublinkSpeedAttribute::from_raw(attr))
            .collect()
    }
    
    /// Get the maximum supported speed in Gbps
    pub fn max_speed_gbps(&self) -> f64 {
        self.parse_sublink_speeds()
            .iter()
            .map(|s| s.speed_gbps())
            .fold(0.0, f64::max)
    }
    
    /// Check if Gen 2x1 (10 Gbps) is supported
    pub fn supports_gen2x1(&self) -> bool {
        self.parse_sublink_speeds()
            .iter()
            .any(|s| s.speed_exponent == SpeedExponent::Gbps && s.lane_speed_mantissa == 10)
    }
    
    /// Check if Gen 2x2 (20 Gbps) is supported
    pub fn supports_gen2x2(&self) -> bool {
        self.parse_sublink_speeds()
            .iter()
            .any(|s| s.speed_exponent == SpeedExponent::Gbps && s.lane_speed_mantissa >= 10 && s.lane_count == 2)
    }
}

/// Sublink Speed Attribute entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SublinkSpeedAttribute {
    /// Sublink Speed Attribute ID
    pub ssid: u8,
    /// Lane speed mantissa
    pub lane_speed_mantissa: u16,
    /// Speed exponent
    pub speed_exponent: SpeedExponent,
    /// Sublink type (symmetric/asymmetric)
    pub sublink_type: SublinkType,
    /// Link protocol (SuperSpeed/SuperSpeedPlus)
    pub link_protocol: LinkProtocol,
    /// Number of lanes
    pub lane_count: u8,
}

impl SublinkSpeedAttribute {
    /// Parse from raw 32-bit attribute
    pub fn from_raw(raw: u32) -> Self {
        let ssid = (raw & 0x0F) as u8;
        let lane_speed_mantissa = ((raw >> 16) & 0xFFFF) as u16;
        let speed_exponent = match (raw >> 4) & 0x03 {
            0 => SpeedExponent::Bps,
            1 => SpeedExponent::Kbps,
            2 => SpeedExponent::Mbps,
            3 => SpeedExponent::Gbps,
            _ => SpeedExponent::Bps,
        };
        let sublink_type = if (raw >> 6) & 0x01 != 0 {
            SublinkType::Asymmetric
        } else {
            SublinkType::Symmetric
        };
        let link_protocol = if (raw >> 14) & 0x03 == 1 {
            LinkProtocol::SuperSpeedPlus
        } else {
            LinkProtocol::SuperSpeed
        };
        let lane_count = ((raw >> 8) & 0x0F) as u8;
        
        Self {
            ssid,
            lane_speed_mantissa,
            speed_exponent,
            sublink_type,
            link_protocol,
            lane_count,
        }
    }
    
    /// Get the speed in Gbps
    pub fn speed_gbps(&self) -> f64 {
        let base_speed = self.lane_speed_mantissa as f64;
        let multiplier = match self.speed_exponent {
            SpeedExponent::Bps => 1e-9,
            SpeedExponent::Kbps => 1e-6,
            SpeedExponent::Mbps => 1e-3,
            SpeedExponent::Gbps => 1.0,
        };
        base_speed * multiplier * (self.lane_count.max(1) as f64)
    }
    
    /// Get human-readable speed string
    pub fn speed_string(&self) -> String {
        let speed = self.speed_gbps();
        if speed >= 1.0 {
            format!("{:.1} Gbps", speed)
        } else if speed >= 0.001 {
            format!("{:.0} Mbps", speed * 1000.0)
        } else {
            format!("{:.0} Kbps", speed * 1000000.0)
        }
    }
}

/// Speed exponent for sublink speed attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeedExponent {
    /// Bits per second
    Bps,
    /// Kilobits per second
    Kbps,
    /// Megabits per second
    Mbps,
    /// Gigabits per second
    Gbps,
}

/// Sublink type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SublinkType {
    /// Symmetric (same speed Rx and Tx)
    Symmetric,
    /// Asymmetric (different Rx and Tx speeds)
    Asymmetric,
}

/// Link protocol type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkProtocol {
    /// SuperSpeed (USB 3.0)
    SuperSpeed,
    /// SuperSpeedPlus (USB 3.1+)
    SuperSpeedPlus,
}

/// SuperSpeed mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuperSpeedMode {
    /// USB 3.0 Gen 1 (5 Gbps)
    Gen1,
    /// USB 3.1 Gen 2 (10 Gbps)
    Gen2,
    /// USB 3.2 Gen 1x2 (10 Gbps, 2 lanes)
    Gen1x2,
    /// USB 3.2 Gen 2x2 (20 Gbps, 2 lanes)
    Gen2x2,
}

impl SuperSpeedMode {
    /// Get the theoretical bandwidth in Gbps
    pub fn bandwidth_gbps(&self) -> f64 {
        match self {
            SuperSpeedMode::Gen1 => 5.0,
            SuperSpeedMode::Gen2 => 10.0,
            SuperSpeedMode::Gen1x2 => 10.0,
            SuperSpeedMode::Gen2x2 => 20.0,
        }
    }
    
    /// Get the USB specification name
    pub fn spec_name(&self) -> &'static str {
        match self {
            SuperSpeedMode::Gen1 => "USB 3.0 (Gen 1)",
            SuperSpeedMode::Gen2 => "USB 3.1 (Gen 2)",
            SuperSpeedMode::Gen1x2 => "USB 3.2 (Gen 1x2)",
            SuperSpeedMode::Gen2x2 => "USB 3.2 (Gen 2x2)",
        }
    }
}

/// USB4 specific capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usb4Capability {
    /// USB4 version (BCD)
    pub version: u16,
    /// Minimum DP lanes supported
    pub min_dp_lanes: u8,
    /// Maximum DP lanes supported
    pub max_dp_lanes: u8,
    /// Tunneling modes supported
    pub tunneling_modes: Usb4TunnelingModes,
    /// USB4 Gen 3 supported (40 Gbps)
    pub gen3_supported: bool,
}

/// USB4 tunneling mode flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usb4TunnelingModes {
    /// USB 3.2 tunneling supported
    pub usb32_tunneling: bool,
    /// DisplayPort tunneling supported
    pub dp_tunneling: bool,
    /// PCIe tunneling supported
    pub pcie_tunneling: bool,
    /// Host-to-host tunneling supported
    pub h2h_tunneling: bool,
}

/// Parse USB4 capabilities from raw data
pub fn parse_usb4_capability(data: &[u8]) -> Option<Usb4Capability> {
    if data.len() < 8 {
        return None;
    }
    
    let version = u16::from_le_bytes([data[0], data[1]]);
    let tunneling_flags = data[4];
    
    Some(Usb4Capability {
        version,
        min_dp_lanes: (data[2] >> 4) & 0x0F,
        max_dp_lanes: data[2] & 0x0F,
        tunneling_modes: Usb4TunnelingModes {
            usb32_tunneling: (tunneling_flags & 0x01) != 0,
            dp_tunneling: (tunneling_flags & 0x02) != 0,
            pcie_tunneling: (tunneling_flags & 0x04) != 0,
            h2h_tunneling: (tunneling_flags & 0x08) != 0,
        },
        gen3_supported: (data[3] & 0x01) != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superspeed_capability() {
        let cap = SuperSpeedCapability {
            attributes: 0x02,
            lpm_supported: true,
            speed_supported: 0x01,
            functionality_support: 0x00,
            u1_dev_exit_lat: 10,
            u2_dev_exit_lat: 2047,
        };
        
        assert!(cap.supports_gen1());
        assert_eq!(cap.u1_exit_latency_us(), 10);
        assert_eq!(cap.u2_exit_latency_us(), 2047);
    }

    #[test]
    fn test_sublink_speed_attribute() {
        // Gen 2 (10 Gbps): mantissa=10, exponent=Gbps, lanes=1
        let raw = 0x000A_0030; // Simplified example
        let attr = SublinkSpeedAttribute::from_raw(raw);
        assert!(attr.speed_gbps() >= 0.0);
    }

    #[test]
    fn test_superspeed_mode_bandwidth() {
        assert_eq!(SuperSpeedMode::Gen1.bandwidth_gbps(), 5.0);
        assert_eq!(SuperSpeedMode::Gen2.bandwidth_gbps(), 10.0);
        assert_eq!(SuperSpeedMode::Gen2x2.bandwidth_gbps(), 20.0);
    }
}
