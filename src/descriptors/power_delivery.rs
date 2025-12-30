//! USB Power Delivery (USB-PD) Detection and Status
//!
//! This module provides USB Power Delivery status detection including:
//! - Power capability detection
//! - Voltage and current negotiation status
//! - Power role (source/sink)
//! - Data role (host/device)
//! - Alternate mode negotiation status

use serde::{Deserialize, Serialize};

/// USB Power Delivery status and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerDeliveryStatus {
    /// Whether USB-PD is supported
    pub pd_supported: bool,
    /// USB-PD specification version
    pub pd_version: Option<PdVersion>,
    /// Current power contract
    pub power_contract: Option<PowerContract>,
    /// Device's power role
    pub power_role: PowerRole,
    /// Device's data role
    pub data_role: DataRole,
    /// Available power profiles (PDOs)
    pub power_profiles: Vec<PowerDataObject>,
    /// Battery capabilities (if applicable)
    pub battery_capabilities: Option<BatteryCapabilities>,
    /// Alternate modes supported
    pub alternate_modes: Vec<AlternateMode>,
}

impl Default for PowerDeliveryStatus {
    fn default() -> Self {
        Self {
            pd_supported: false,
            pd_version: None,
            power_contract: None,
            power_role: PowerRole::Unknown,
            data_role: DataRole::Unknown,
            power_profiles: Vec::new(),
            battery_capabilities: None,
            alternate_modes: Vec::new(),
        }
    }
}

/// USB-PD specification version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PdVersion {
    /// PD 1.0
    Pd1_0,
    /// PD 2.0
    Pd2_0,
    /// PD 3.0
    Pd3_0,
    /// PD 3.1 (Extended Power Range)
    Pd3_1,
    /// Unknown version
    Unknown(u8),
}

impl PdVersion {
    /// Parse from version bytes
    pub fn from_version(major: u8, minor: u8) -> Self {
        match (major, minor) {
            (1, 0) => PdVersion::Pd1_0,
            (2, 0) => PdVersion::Pd2_0,
            (3, 0) => PdVersion::Pd3_0,
            (3, 1) => PdVersion::Pd3_1,
            _ => PdVersion::Unknown(major),
        }
    }
    
    /// Check if Extended Power Range (EPR) is supported
    pub fn supports_epr(&self) -> bool {
        matches!(self, PdVersion::Pd3_1)
    }
}

/// Current power contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerContract {
    /// Negotiated voltage in millivolts
    pub voltage_mv: u32,
    /// Negotiated current in milliamps
    pub current_ma: u32,
    /// Power in milliwatts
    pub power_mw: u32,
    /// PDO index that was selected
    pub selected_pdo_index: u8,
    /// Whether this is a PPS (Programmable Power Supply) contract
    pub is_pps: bool,
}

impl PowerContract {
    /// Calculate power in watts
    pub fn power_watts(&self) -> f64 {
        self.power_mw as f64 / 1000.0
    }
}

/// Power role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerRole {
    /// Power source (providing power)
    Source,
    /// Power sink (consuming power)
    Sink,
    /// Dual-role power
    DualRole,
    /// Unknown
    Unknown,
}

/// Data role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataRole {
    /// Host (DFP - Downstream Facing Port)
    Host,
    /// Device (UFP - Upstream Facing Port)
    Device,
    /// Dual-role data
    DualRole,
    /// Unknown
    Unknown,
}

/// Power Data Object (PDO) - describes a power profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PowerDataObject {
    /// Fixed Supply PDO
    FixedSupply(FixedSupplyPdo),
    /// Variable Supply PDO
    VariableSupply(VariableSupplyPdo),
    /// Battery PDO
    Battery(BatteryPdo),
    /// Programmable Power Supply (PPS) PDO
    Pps(PpsPdo),
    /// Extended Power Range (EPR) Fixed Supply PDO
    EprFixedSupply(EprFixedSupplyPdo),
    /// Extended Power Range (EPR) Adjustable Voltage Supply PDO
    EprAdjustableVoltage(EprAdjustableVoltagePdo),
    /// Unknown PDO type
    Unknown(u32),
}

impl PowerDataObject {
    /// Parse PDO from raw 32-bit value
    pub fn from_raw(raw: u32) -> Self {
        let pdo_type = (raw >> 30) & 0x03;
        
        match pdo_type {
            0 => Self::parse_fixed_supply(raw),
            1 => Self::parse_battery(raw),
            2 => Self::parse_variable_supply(raw),
            3 => Self::parse_augmented(raw),
            _ => PowerDataObject::Unknown(raw),
        }
    }
    
    fn parse_fixed_supply(raw: u32) -> Self {
        PowerDataObject::FixedSupply(FixedSupplyPdo {
            voltage_mv: ((raw >> 10) & 0x3FF) * 50,
            max_current_ma: (raw & 0x3FF) * 10,
            dual_role_power: (raw >> 29) & 0x01 != 0,
            usb_suspend_supported: (raw >> 28) & 0x01 != 0,
            unconstrained_power: (raw >> 27) & 0x01 != 0,
            usb_comm_capable: (raw >> 26) & 0x01 != 0,
            dual_role_data: (raw >> 25) & 0x01 != 0,
            unchunked_extended_messages: (raw >> 24) & 0x01 != 0,
            epr_capable: (raw >> 23) & 0x01 != 0,
            peak_current: ((raw >> 20) & 0x03) as u8,
        })
    }
    
    fn parse_battery(raw: u32) -> Self {
        PowerDataObject::Battery(BatteryPdo {
            max_voltage_mv: ((raw >> 20) & 0x3FF) * 50,
            min_voltage_mv: ((raw >> 10) & 0x3FF) * 50,
            max_power_mw: (raw & 0x3FF) * 250,
        })
    }
    
    fn parse_variable_supply(raw: u32) -> Self {
        PowerDataObject::VariableSupply(VariableSupplyPdo {
            max_voltage_mv: ((raw >> 20) & 0x3FF) * 50,
            min_voltage_mv: ((raw >> 10) & 0x3FF) * 50,
            max_current_ma: (raw & 0x3FF) * 10,
        })
    }
    
    fn parse_augmented(raw: u32) -> Self {
        let apdo_type = (raw >> 28) & 0x03;
        
        match apdo_type {
            0 => PowerDataObject::Pps(PpsPdo {
                max_voltage_mv: ((raw >> 17) & 0xFF) * 100,
                min_voltage_mv: ((raw >> 8) & 0xFF) * 100,
                max_current_ma: (raw & 0x7F) * 50,
                pps_power_limited: (raw >> 27) & 0x01 != 0,
            }),
            1 => PowerDataObject::EprAdjustableVoltage(EprAdjustableVoltagePdo {
                max_voltage_mv: ((raw >> 17) & 0x1FF) * 100,
                min_voltage_mv: ((raw >> 8) & 0xFF) * 100,
                max_power_mw: (raw & 0xFF) * 1000,
            }),
            _ => PowerDataObject::Unknown(raw),
        }
    }
    
    /// Get the maximum power this PDO can deliver in milliwatts
    pub fn max_power_mw(&self) -> u32 {
        match self {
            PowerDataObject::FixedSupply(pdo) => pdo.voltage_mv * pdo.max_current_ma / 1000,
            PowerDataObject::VariableSupply(pdo) => pdo.max_voltage_mv * pdo.max_current_ma / 1000,
            PowerDataObject::Battery(pdo) => pdo.max_power_mw,
            PowerDataObject::Pps(pdo) => pdo.max_voltage_mv * pdo.max_current_ma / 1000,
            PowerDataObject::EprFixedSupply(pdo) => pdo.voltage_mv * pdo.max_current_ma / 1000,
            PowerDataObject::EprAdjustableVoltage(pdo) => pdo.max_power_mw,
            PowerDataObject::Unknown(_) => 0,
        }
    }
    
    /// Get the voltage(s) this PDO provides
    pub fn voltage_description(&self) -> String {
        match self {
            PowerDataObject::FixedSupply(pdo) => format!("{}V", pdo.voltage_mv as f64 / 1000.0),
            PowerDataObject::VariableSupply(pdo) => {
                format!("{}-{}V", pdo.min_voltage_mv as f64 / 1000.0, pdo.max_voltage_mv as f64 / 1000.0)
            }
            PowerDataObject::Battery(pdo) => {
                format!("{}-{}V (Battery)", pdo.min_voltage_mv as f64 / 1000.0, pdo.max_voltage_mv as f64 / 1000.0)
            }
            PowerDataObject::Pps(pdo) => {
                format!("{}-{}V PPS", pdo.min_voltage_mv as f64 / 1000.0, pdo.max_voltage_mv as f64 / 1000.0)
            }
            PowerDataObject::EprFixedSupply(pdo) => format!("{}V EPR", pdo.voltage_mv as f64 / 1000.0),
            PowerDataObject::EprAdjustableVoltage(pdo) => {
                format!("{}-{}V EPR AVS", pdo.min_voltage_mv as f64 / 1000.0, pdo.max_voltage_mv as f64 / 1000.0)
            }
            PowerDataObject::Unknown(_) => "Unknown".to_string(),
        }
    }
}

/// Fixed Supply PDO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedSupplyPdo {
    /// Voltage in millivolts
    pub voltage_mv: u32,
    /// Maximum current in milliamps
    pub max_current_ma: u32,
    /// Dual-role power capable
    pub dual_role_power: bool,
    /// USB suspend supported
    pub usb_suspend_supported: bool,
    /// Unconstrained power
    pub unconstrained_power: bool,
    /// USB communications capable
    pub usb_comm_capable: bool,
    /// Dual-role data capable
    pub dual_role_data: bool,
    /// Unchunked extended messages supported
    pub unchunked_extended_messages: bool,
    /// EPR mode capable (PD 3.1)
    pub epr_capable: bool,
    /// Peak current capability (0-3)
    pub peak_current: u8,
}

impl FixedSupplyPdo {
    /// Get power in watts
    pub fn power_watts(&self) -> f64 {
        (self.voltage_mv as f64 / 1000.0) * (self.max_current_ma as f64 / 1000.0)
    }
}

/// Variable Supply PDO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableSupplyPdo {
    /// Maximum voltage in millivolts
    pub max_voltage_mv: u32,
    /// Minimum voltage in millivolts
    pub min_voltage_mv: u32,
    /// Maximum current in milliamps
    pub max_current_ma: u32,
}

/// Battery PDO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryPdo {
    /// Maximum voltage in millivolts
    pub max_voltage_mv: u32,
    /// Minimum voltage in millivolts
    pub min_voltage_mv: u32,
    /// Maximum power in milliwatts
    pub max_power_mw: u32,
}

/// Programmable Power Supply (PPS) PDO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PpsPdo {
    /// Maximum voltage in millivolts
    pub max_voltage_mv: u32,
    /// Minimum voltage in millivolts
    pub min_voltage_mv: u32,
    /// Maximum current in milliamps
    pub max_current_ma: u32,
    /// PPS power limited flag
    pub pps_power_limited: bool,
}

/// Extended Power Range (EPR) Fixed Supply PDO (PD 3.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EprFixedSupplyPdo {
    /// Voltage in millivolts (up to 48V)
    pub voltage_mv: u32,
    /// Maximum current in milliamps (up to 5A)
    pub max_current_ma: u32,
    /// Peak current capability
    pub peak_current: u8,
}

/// Extended Power Range (EPR) Adjustable Voltage Supply PDO (PD 3.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EprAdjustableVoltagePdo {
    /// Maximum voltage in millivolts
    pub max_voltage_mv: u32,
    /// Minimum voltage in millivolts
    pub min_voltage_mv: u32,
    /// Maximum power in milliwatts
    pub max_power_mw: u32,
}

/// Battery capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryCapabilities {
    /// Battery present
    pub present: bool,
    /// State of charge (0-100%)
    pub state_of_charge: Option<u8>,
    /// Battery voltage in millivolts
    pub voltage_mv: Option<u32>,
    /// Battery design capacity in mWh
    pub design_capacity_mwh: Option<u32>,
    /// Battery last full charge capacity in mWh
    pub last_full_capacity_mwh: Option<u32>,
    /// Battery type
    pub battery_type: Option<String>,
}

/// Alternate Mode information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternateMode {
    /// Standard or Vendor ID (SVID)
    pub svid: u16,
    /// SVID name (if known)
    pub svid_name: String,
    /// Mode VDO (Vendor Defined Object)
    pub mode_vdo: u32,
    /// Mode index
    pub mode_index: u8,
    /// Whether this mode is currently active
    pub active: bool,
}

impl AlternateMode {
    /// Check if this is DisplayPort alternate mode
    pub fn is_displayport(&self) -> bool {
        self.svid == 0xFF01
    }
    
    /// Check if this is Thunderbolt alternate mode
    pub fn is_thunderbolt(&self) -> bool {
        self.svid == 0x8087
    }
    
    /// Get the known SVID name
    pub fn get_svid_name(svid: u16) -> &'static str {
        match svid {
            0xFF01 => "DisplayPort",
            0x8087 => "Thunderbolt",
            0x05AC => "Apple",
            0x18D1 => "Google",
            0x04B4 => "Cypress",
            0x1D6B => "Linux Foundation",
            _ => "Unknown",
        }
    }
}

/// Well-known SVIDs (Standard/Vendor IDs)
pub mod svid {
    /// DisplayPort alternate mode SVID
    pub const DISPLAYPORT: u16 = 0xFF01;
    /// Thunderbolt alternate mode SVID
    pub const THUNDERBOLT: u16 = 0x8087;
    /// Apple alternate mode SVID
    pub const APPLE: u16 = 0x05AC;
    /// Google alternate mode SVID
    pub const GOOGLE: u16 = 0x18D1;
}

/// Common USB-PD power profiles
pub mod common_profiles {
    use super::*;
    
    /// 5V @ 3A (15W) - USB-C baseline
    pub fn usb_c_baseline() -> FixedSupplyPdo {
        FixedSupplyPdo {
            voltage_mv: 5000,
            max_current_ma: 3000,
            dual_role_power: false,
            usb_suspend_supported: false,
            unconstrained_power: false,
            usb_comm_capable: true,
            dual_role_data: false,
            unchunked_extended_messages: false,
            epr_capable: false,
            peak_current: 0,
        }
    }
    
    /// 9V @ 3A (27W)
    pub fn pd_27w() -> FixedSupplyPdo {
        FixedSupplyPdo {
            voltage_mv: 9000,
            max_current_ma: 3000,
            ..usb_c_baseline()
        }
    }
    
    /// 15V @ 3A (45W)
    pub fn pd_45w() -> FixedSupplyPdo {
        FixedSupplyPdo {
            voltage_mv: 15000,
            max_current_ma: 3000,
            ..usb_c_baseline()
        }
    }
    
    /// 20V @ 3A (60W)
    pub fn pd_60w() -> FixedSupplyPdo {
        FixedSupplyPdo {
            voltage_mv: 20000,
            max_current_ma: 3000,
            ..usb_c_baseline()
        }
    }
    
    /// 20V @ 5A (100W) - Maximum standard PD
    pub fn pd_100w() -> FixedSupplyPdo {
        FixedSupplyPdo {
            voltage_mv: 20000,
            max_current_ma: 5000,
            ..usb_c_baseline()
        }
    }
    
    /// 28V @ 5A (140W) - EPR
    pub fn pd_140w_epr() -> EprFixedSupplyPdo {
        EprFixedSupplyPdo {
            voltage_mv: 28000,
            max_current_ma: 5000,
            peak_current: 0,
        }
    }
    
    /// 48V @ 5A (240W) - Maximum EPR
    pub fn pd_240w_epr() -> EprFixedSupplyPdo {
        EprFixedSupplyPdo {
            voltage_mv: 48000,
            max_current_ma: 5000,
            peak_current: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_supply_pdo_parsing() {
        // 5V @ 3A fixed supply
        let raw = 0x0001912C; // Example encoding
        let pdo = PowerDataObject::from_raw(raw);
        assert!(matches!(pdo, PowerDataObject::FixedSupply(_)));
    }

    #[test]
    fn test_pdo_power_calculation() {
        let pdo = FixedSupplyPdo {
            voltage_mv: 20000,
            max_current_ma: 5000,
            dual_role_power: false,
            usb_suspend_supported: false,
            unconstrained_power: false,
            usb_comm_capable: false,
            dual_role_data: false,
            unchunked_extended_messages: false,
            epr_capable: false,
            peak_current: 0,
        };
        
        assert_eq!(pdo.power_watts(), 100.0);
    }

    #[test]
    fn test_alternate_mode_detection() {
        let dp_mode = AlternateMode {
            svid: 0xFF01,
            svid_name: "DisplayPort".to_string(),
            mode_vdo: 0,
            mode_index: 0,
            active: false,
        };
        
        assert!(dp_mode.is_displayport());
        assert!(!dp_mode.is_thunderbolt());
    }
}
