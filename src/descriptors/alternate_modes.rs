//! USB Type-C Alternate Mode Detection
//!
//! This module provides detection and information for USB Type-C alternate modes:
//! - DisplayPort Alternate Mode
//! - Thunderbolt Alternate Mode
//! - HDMI Alternate Mode (rare)
//! - Vendor-specific alternate modes

use super::power_delivery::{AlternateMode, svid};
use serde::{Deserialize, Serialize};

/// DisplayPort Alternate Mode capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayPortAltMode {
    /// DP version supported
    pub dp_version: DpVersion,
    /// Maximum supported resolution
    pub max_resolution: DpResolution,
    /// Maximum refresh rate at max resolution
    pub max_refresh_hz: u32,
    /// Pin assignment supported
    pub pin_assignments: Vec<DpPinAssignment>,
    /// Multi-function preferred (DP + USB data)
    pub multi_function_preferred: bool,
    /// USB 2.0 signaling maintained
    pub usb2_signaling: bool,
    /// Receptacle indication
    pub receptacle: bool,
    /// DFP_D/UFP_D capable
    pub dfp_d_capable: bool,
    pub ufp_d_capable: bool,
}

/// DisplayPort version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DpVersion {
    /// DisplayPort 1.1
    Dp1_1,
    /// DisplayPort 1.2
    Dp1_2,
    /// DisplayPort 1.3
    Dp1_3,
    /// DisplayPort 1.4
    Dp1_4,
    /// DisplayPort 2.0
    Dp2_0,
    /// DisplayPort 2.1
    Dp2_1,
    /// Unknown version
    Unknown(u8),
}

impl DpVersion {
    /// Get the maximum bandwidth in Gbps
    pub fn max_bandwidth_gbps(&self) -> f64 {
        match self {
            DpVersion::Dp1_1 => 2.7 * 4.0, // 4 lanes
            DpVersion::Dp1_2 => 5.4 * 4.0,
            DpVersion::Dp1_3 => 8.1 * 4.0,
            DpVersion::Dp1_4 => 8.1 * 4.0, // Same as 1.3 but with DSC
            DpVersion::Dp2_0 => 20.0 * 4.0,
            DpVersion::Dp2_1 => 20.0 * 4.0,
            DpVersion::Unknown(_) => 0.0,
        }
    }
    
    /// Check if DSC (Display Stream Compression) is supported
    pub fn supports_dsc(&self) -> bool {
        matches!(self, DpVersion::Dp1_4 | DpVersion::Dp2_0 | DpVersion::Dp2_1)
    }
}

/// DisplayPort resolution capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpResolution {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Bits per pixel
    pub bpp: u8,
}

impl DpResolution {
    pub fn _4k() -> Self {
        Self { width: 3840, height: 2160, bpp: 24 }
    }
    
    pub fn _8k() -> Self {
        Self { width: 7680, height: 4320, bpp: 24 }
    }
    
    pub fn _1080p() -> Self {
        Self { width: 1920, height: 1080, bpp: 24 }
    }
    
    /// Get the name of this resolution
    pub fn name(&self) -> &'static str {
        match (self.width, self.height) {
            (1920, 1080) => "1080p",
            (2560, 1440) => "1440p",
            (3840, 2160) => "4K",
            (5120, 2880) => "5K",
            (7680, 4320) => "8K",
            _ => "Custom",
        }
    }
}

/// DisplayPort pin assignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DpPinAssignment {
    /// Pin Assignment A: 4 lanes DP, no USB 3.x
    A,
    /// Pin Assignment B: 4 lanes DP, no USB 3.x (alternate)
    B,
    /// Pin Assignment C: 4 lanes DP, no USB 3.x (for USB-C to DP cables)
    C,
    /// Pin Assignment D: 2 lanes DP + USB 3.x
    D,
    /// Pin Assignment E: 4 lanes DP (for DP to USB-C adapters)
    E,
    /// Pin Assignment F: 2 lanes DP + USB 3.x (for DP to USB-C adapters)
    F,
}

impl DpPinAssignment {
    /// Get number of DP lanes
    pub fn dp_lanes(&self) -> u8 {
        match self {
            DpPinAssignment::A | DpPinAssignment::B | DpPinAssignment::C | DpPinAssignment::E => 4,
            DpPinAssignment::D | DpPinAssignment::F => 2,
        }
    }
    
    /// Check if USB 3.x data is supported simultaneously
    pub fn usb3_supported(&self) -> bool {
        matches!(self, DpPinAssignment::D | DpPinAssignment::F)
    }
}

/// Thunderbolt Alternate Mode capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThunderboltAltMode {
    /// Thunderbolt version
    pub tb_version: TbVersion,
    /// Maximum data bandwidth
    pub max_data_bandwidth_gbps: f64,
    /// PCIe tunneling supported
    pub pcie_tunneling: bool,
    /// DisplayPort tunneling supported
    pub dp_tunneling: bool,
    /// USB 3.x tunneling supported
    pub usb3_tunneling: bool,
    /// Vendor-specific extended mode info
    pub vendor_mode: Option<u32>,
}

/// Thunderbolt version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TbVersion {
    /// Thunderbolt 3 (40 Gbps)
    Tb3,
    /// Thunderbolt 4 (40 Gbps, but with stricter requirements)
    Tb4,
    /// Thunderbolt 5 (80 Gbps, 120 Gbps asymmetric)
    Tb5,
    /// Unknown version
    Unknown(u8),
}

impl TbVersion {
    /// Get the maximum bandwidth in Gbps
    pub fn max_bandwidth_gbps(&self) -> f64 {
        match self {
            TbVersion::Tb3 => 40.0,
            TbVersion::Tb4 => 40.0,
            TbVersion::Tb5 => 80.0, // 120 Gbps asymmetric
            TbVersion::Unknown(_) => 0.0,
        }
    }
    
    /// Get minimum PCIe tunneling bandwidth
    pub fn min_pcie_bandwidth_gbps(&self) -> f64 {
        match self {
            TbVersion::Tb3 => 22.0,
            TbVersion::Tb4 => 32.0, // Stricter requirements
            TbVersion::Tb5 => 64.0,
            TbVersion::Unknown(_) => 0.0,
        }
    }
}

/// All detected alternate modes for a device
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlternateModeCapabilities {
    /// DisplayPort alternate mode (if supported)
    pub displayport: Option<DisplayPortAltMode>,
    /// Thunderbolt alternate mode (if supported)
    pub thunderbolt: Option<ThunderboltAltMode>,
    /// Other vendor-specific alternate modes
    pub other_modes: Vec<AlternateMode>,
}

impl AlternateModeCapabilities {
    /// Check if any video output is supported
    pub fn supports_video_output(&self) -> bool {
        self.displayport.is_some() || self.thunderbolt.as_ref().map_or(false, |t| t.dp_tunneling)
    }
    
    /// Check if Thunderbolt is supported
    pub fn supports_thunderbolt(&self) -> bool {
        self.thunderbolt.is_some()
    }
    
    /// Get the maximum video bandwidth available
    pub fn max_video_bandwidth_gbps(&self) -> f64 {
        let dp_bandwidth = self.displayport.as_ref()
            .map(|d| d.dp_version.max_bandwidth_gbps())
            .unwrap_or(0.0);
            
        let tb_bandwidth = self.thunderbolt.as_ref()
            .filter(|t| t.dp_tunneling)
            .map(|t| t.max_data_bandwidth_gbps)
            .unwrap_or(0.0);
            
        dp_bandwidth.max(tb_bandwidth)
    }
}

/// Parse DisplayPort VDO (Vendor Defined Object) from mode_vdo
pub fn parse_dp_vdo(mode_vdo: u32) -> DisplayPortAltMode {
    // DP VDO format (USB Type-C specification):
    // Bits 0-1: Port capability (00=reserved, 01=UFP_D, 10=DFP_D, 11=both)
    // Bits 2-7: Signaling for Transport (HBR3, HBR2, etc.)
    // Bits 8-15: Pin assignment supported (bitmap)
    // Bit 24: USB 2.0 signaling not used
    // Bit 25: Receptacle indication
    // Bit 26: Multi-function preferred
    
    let port_cap = mode_vdo & 0x03;
    let dfp_d_capable = (port_cap & 0x02) != 0;
    let ufp_d_capable = (port_cap & 0x01) != 0;
    
    let signaling = (mode_vdo >> 2) & 0x3F;
    let dp_version = if (signaling & 0x20) != 0 {
        DpVersion::Dp2_0
    } else if (signaling & 0x10) != 0 {
        DpVersion::Dp1_4
    } else if (signaling & 0x08) != 0 {
        DpVersion::Dp1_3
    } else if (signaling & 0x04) != 0 {
        DpVersion::Dp1_2
    } else {
        DpVersion::Dp1_1
    };
    
    let pin_mask = (mode_vdo >> 8) & 0xFF;
    let mut pin_assignments = Vec::new();
    if (pin_mask & 0x01) != 0 { pin_assignments.push(DpPinAssignment::A); }
    if (pin_mask & 0x02) != 0 { pin_assignments.push(DpPinAssignment::B); }
    if (pin_mask & 0x04) != 0 { pin_assignments.push(DpPinAssignment::C); }
    if (pin_mask & 0x08) != 0 { pin_assignments.push(DpPinAssignment::D); }
    if (pin_mask & 0x10) != 0 { pin_assignments.push(DpPinAssignment::E); }
    if (pin_mask & 0x20) != 0 { pin_assignments.push(DpPinAssignment::F); }
    
    let usb2_signaling = (mode_vdo >> 24) & 0x01 == 0;
    let receptacle = (mode_vdo >> 25) & 0x01 != 0;
    let multi_function_preferred = (mode_vdo >> 26) & 0x01 != 0;
    
    // Determine max resolution based on DP version
    let (max_resolution, max_refresh_hz) = match dp_version {
        DpVersion::Dp1_1 => (DpResolution::_1080p(), 60),
        DpVersion::Dp1_2 => (DpResolution::_4k(), 60),
        DpVersion::Dp1_3 | DpVersion::Dp1_4 => (DpResolution::_8k(), 30),
        DpVersion::Dp2_0 | DpVersion::Dp2_1 => (DpResolution::_8k(), 60),
        DpVersion::Unknown(_) => (DpResolution::_1080p(), 60),
    };
    
    DisplayPortAltMode {
        dp_version,
        max_resolution,
        max_refresh_hz,
        pin_assignments,
        multi_function_preferred,
        usb2_signaling,
        receptacle,
        dfp_d_capable,
        ufp_d_capable,
    }
}

/// Parse Thunderbolt VDO from mode_vdo
pub fn parse_tb_vdo(mode_vdo: u32) -> ThunderboltAltMode {
    // Thunderbolt VDO format:
    // Bits 0-15: Vendor-specific
    // Bits 16-19: Thunderbolt adapter type
    // Bits 20-22: Cable speed
    // Bit 25: Active cable
    // Bit 26: Retimer included
    
    let cable_speed = (mode_vdo >> 20) & 0x07;
    let (tb_version, max_data_bandwidth_gbps) = match cable_speed {
        0 => (TbVersion::Tb3, 20.0),
        1 => (TbVersion::Tb3, 40.0),
        2 => (TbVersion::Tb4, 40.0),
        3 => (TbVersion::Tb5, 80.0),
        _ => (TbVersion::Unknown(cable_speed as u8), 0.0),
    };
    
    ThunderboltAltMode {
        tb_version,
        max_data_bandwidth_gbps,
        pcie_tunneling: true, // Always supported
        dp_tunneling: true,   // Always supported
        usb3_tunneling: true, // Always supported
        vendor_mode: Some(mode_vdo),
    }
}

/// Detect alternate mode capabilities from a list of alternate modes
pub fn detect_alternate_modes(modes: &[AlternateMode]) -> AlternateModeCapabilities {
    let mut caps = AlternateModeCapabilities::default();
    
    for mode in modes {
        if mode.svid == svid::DISPLAYPORT {
            caps.displayport = Some(parse_dp_vdo(mode.mode_vdo));
        } else if mode.svid == svid::THUNDERBOLT {
            caps.thunderbolt = Some(parse_tb_vdo(mode.mode_vdo));
        } else {
            caps.other_modes.push(mode.clone());
        }
    }
    
    caps
}

/// Known vendor SVIDs for alternate modes
pub mod known_svids {
    /// DisplayPort
    pub const DISPLAYPORT: u16 = 0xFF01;
    /// Thunderbolt (Intel)
    pub const THUNDERBOLT: u16 = 0x8087;
    /// HDMI (HDMI Forum)
    pub const HDMI: u16 = 0xFF02;
    /// Apple
    pub const APPLE: u16 = 0x05AC;
    /// Google
    pub const GOOGLE: u16 = 0x18D1;
    /// Samsung
    pub const SAMSUNG: u16 = 0x04E8;
    /// Microsoft
    pub const MICROSOFT: u16 = 0x045E;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dp_version_bandwidth() {
        assert!(DpVersion::Dp1_4.max_bandwidth_gbps() > DpVersion::Dp1_2.max_bandwidth_gbps());
        assert!(DpVersion::Dp2_0.supports_dsc());
        assert!(!DpVersion::Dp1_2.supports_dsc());
    }

    #[test]
    fn test_pin_assignment() {
        assert_eq!(DpPinAssignment::A.dp_lanes(), 4);
        assert_eq!(DpPinAssignment::D.dp_lanes(), 2);
        assert!(DpPinAssignment::D.usb3_supported());
        assert!(!DpPinAssignment::A.usb3_supported());
    }

    #[test]
    fn test_tb_version_bandwidth() {
        assert_eq!(TbVersion::Tb3.max_bandwidth_gbps(), 40.0);
        assert_eq!(TbVersion::Tb5.max_bandwidth_gbps(), 80.0);
    }

    #[test]
    fn test_dp_vdo_parsing() {
        // Simple VDO with DFP_D + UFP_D, HBR2 signaling, pin assignment D
        let vdo = 0x0408_0003u32;
        let dp = parse_dp_vdo(vdo);
        assert!(dp.dfp_d_capable);
        assert!(dp.ufp_d_capable);
    }
}
