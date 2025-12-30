//! HID (Human Interface Device) Report Descriptor Parser - OMEGA MODE
//!
//! Full implementation of HID report descriptor parsing including:
//! - Main items (Input, Output, Feature, Collection, End Collection)
//! - Global items (Usage Page, Logical Min/Max, Physical Min/Max, etc.)
//! - Local items (Usage, Usage Min/Max, String Index, etc.)
//! - Report structure extraction
//! - Usage page definitions

use std::collections::HashMap;

/// HID item types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    /// Main item
    Main,
    /// Global item
    Global,
    /// Local item
    Local,
    /// Reserved
    Reserved,
}

/// HID main item tags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainTag {
    Input,
    Output,
    Feature,
    Collection,
    EndCollection,
}

/// HID global item tags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalTag {
    UsagePage,
    LogicalMinimum,
    LogicalMaximum,
    PhysicalMinimum,
    PhysicalMaximum,
    UnitExponent,
    Unit,
    ReportSize,
    ReportId,
    ReportCount,
    Push,
    Pop,
}

/// HID local item tags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalTag {
    Usage,
    UsageMinimum,
    UsageMaximum,
    DesignatorIndex,
    DesignatorMinimum,
    DesignatorMaximum,
    StringIndex,
    StringMinimum,
    StringMaximum,
    Delimiter,
}

/// Collection types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionType {
    Physical,
    Application,
    Logical,
    Report,
    NamedArray,
    UsageSwitch,
    UsageModifier,
    VendorDefined(u8),
}

impl CollectionType {
    pub fn from_byte(b: u8) -> Self {
        match b {
            0x00 => Self::Physical,
            0x01 => Self::Application,
            0x02 => Self::Logical,
            0x03 => Self::Report,
            0x04 => Self::NamedArray,
            0x05 => Self::UsageSwitch,
            0x06 => Self::UsageModifier,
            _ => Self::VendorDefined(b),
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Self::Physical => "Physical",
            Self::Application => "Application",
            Self::Logical => "Logical",
            Self::Report => "Report",
            Self::NamedArray => "Named Array",
            Self::UsageSwitch => "Usage Switch",
            Self::UsageModifier => "Usage Modifier",
            Self::VendorDefined(_) => "Vendor Defined",
        }
    }
}

/// Common HID usage pages
pub mod usage_page {
    pub const GENERIC_DESKTOP: u16 = 0x01;
    pub const SIMULATION: u16 = 0x02;
    pub const VR: u16 = 0x03;
    pub const SPORT: u16 = 0x04;
    pub const GAME: u16 = 0x05;
    pub const GENERIC_DEVICE: u16 = 0x06;
    pub const KEYBOARD: u16 = 0x07;
    pub const LED: u16 = 0x08;
    pub const BUTTON: u16 = 0x09;
    pub const ORDINAL: u16 = 0x0A;
    pub const TELEPHONY: u16 = 0x0B;
    pub const CONSUMER: u16 = 0x0C;
    pub const DIGITIZER: u16 = 0x0D;
    pub const HAPTICS: u16 = 0x0E;
    pub const PID: u16 = 0x0F;
    pub const UNICODE: u16 = 0x10;
    pub const EYE_HEAD_TRACKER: u16 = 0x12;
    pub const ALPHANUMERIC_DISPLAY: u16 = 0x14;
    pub const SENSOR: u16 = 0x20;
    pub const MEDICAL: u16 = 0x40;
    pub const MONITOR: u16 = 0x80;
    pub const POWER: u16 = 0x84;
    pub const BAR_CODE_SCANNER: u16 = 0x8C;
    pub const SCALE: u16 = 0x8D;
    pub const MSR: u16 = 0x8E;
    pub const CAMERA: u16 = 0x90;
    pub const ARCADE: u16 = 0x91;
    pub const GAMING_DEVICE: u16 = 0x92;
    pub const FIDO: u16 = 0xF1D0;
    pub const VENDOR_MIN: u16 = 0xFF00;
    pub const VENDOR_MAX: u16 = 0xFFFF;
    
    pub fn name(page: u16) -> &'static str {
        match page {
            GENERIC_DESKTOP => "Generic Desktop",
            SIMULATION => "Simulation",
            VR => "VR",
            SPORT => "Sport",
            GAME => "Game",
            GENERIC_DEVICE => "Generic Device",
            KEYBOARD => "Keyboard/Keypad",
            LED => "LED",
            BUTTON => "Button",
            ORDINAL => "Ordinal",
            TELEPHONY => "Telephony",
            CONSUMER => "Consumer",
            DIGITIZER => "Digitizer",
            HAPTICS => "Haptics",
            PID => "Physical Interface Device",
            UNICODE => "Unicode",
            EYE_HEAD_TRACKER => "Eye/Head Tracker",
            ALPHANUMERIC_DISPLAY => "Alphanumeric Display",
            SENSOR => "Sensor",
            MEDICAL => "Medical",
            MONITOR => "Monitor",
            POWER => "Power",
            BAR_CODE_SCANNER => "Bar Code Scanner",
            SCALE => "Scale",
            MSR => "Magnetic Stripe Reader",
            CAMERA => "Camera",
            ARCADE => "Arcade",
            GAMING_DEVICE => "Gaming Device",
            FIDO => "FIDO Alliance",
            0xFF00..=0xFFFF => "Vendor Defined",
            _ => "Unknown",
        }
    }
}

/// Generic Desktop usages
pub mod usage_desktop {
    pub const POINTER: u16 = 0x01;
    pub const MOUSE: u16 = 0x02;
    pub const JOYSTICK: u16 = 0x04;
    pub const GAMEPAD: u16 = 0x05;
    pub const KEYBOARD: u16 = 0x06;
    pub const KEYPAD: u16 = 0x07;
    pub const MULTI_AXIS: u16 = 0x08;
    pub const TABLET_PC: u16 = 0x09;
    pub const X: u16 = 0x30;
    pub const Y: u16 = 0x31;
    pub const Z: u16 = 0x32;
    pub const RX: u16 = 0x33;
    pub const RY: u16 = 0x34;
    pub const RZ: u16 = 0x35;
    pub const SLIDER: u16 = 0x36;
    pub const DIAL: u16 = 0x37;
    pub const WHEEL: u16 = 0x38;
    pub const HAT_SWITCH: u16 = 0x39;
    pub const DPAD_UP: u16 = 0x90;
    pub const DPAD_DOWN: u16 = 0x91;
    pub const DPAD_RIGHT: u16 = 0x92;
    pub const DPAD_LEFT: u16 = 0x93;
    
    pub fn name(usage: u16) -> &'static str {
        match usage {
            POINTER => "Pointer",
            MOUSE => "Mouse",
            JOYSTICK => "Joystick",
            GAMEPAD => "Gamepad",
            KEYBOARD => "Keyboard",
            KEYPAD => "Keypad",
            MULTI_AXIS => "Multi-axis Controller",
            TABLET_PC => "Tablet PC",
            X => "X",
            Y => "Y",
            Z => "Z",
            RX => "Rx",
            RY => "Ry",
            RZ => "Rz",
            SLIDER => "Slider",
            DIAL => "Dial",
            WHEEL => "Wheel",
            HAT_SWITCH => "Hat Switch",
            DPAD_UP => "D-pad Up",
            DPAD_DOWN => "D-pad Down",
            DPAD_RIGHT => "D-pad Right",
            DPAD_LEFT => "D-pad Left",
            _ => "Unknown",
        }
    }
}

/// Consumer usages
pub mod usage_consumer {
    pub const CONSUMER_CONTROL: u16 = 0x01;
    pub const PLAY_PAUSE: u16 = 0xCD;
    pub const SCAN_NEXT: u16 = 0xB5;
    pub const SCAN_PREVIOUS: u16 = 0xB6;
    pub const STOP: u16 = 0xB7;
    pub const VOLUME: u16 = 0xE0;
    pub const MUTE: u16 = 0xE2;
    pub const VOLUME_UP: u16 = 0xE9;
    pub const VOLUME_DOWN: u16 = 0xEA;
}

/// Parsed HID item
#[derive(Debug, Clone)]
pub struct HidItem {
    /// Item type
    pub item_type: ItemType,
    /// Tag value
    pub tag: u8,
    /// Data size (0, 1, 2, or 4 bytes)
    pub size: u8,
    /// Data value
    pub data: i32,
    /// Raw bytes
    pub raw: Vec<u8>,
}

impl HidItem {
    /// Parse an item from bytes
    pub fn parse(data: &[u8]) -> Option<(Self, usize)> {
        if data.is_empty() {
            return None;
        }
        
        let prefix = data[0];
        
        // Long item format
        if prefix == 0xFE {
            if data.len() < 3 {
                return None;
            }
            let size = data[1] as usize;
            let tag = data[2];
            if data.len() < 3 + size {
                return None;
            }
            return Some((
                Self {
                    item_type: ItemType::Reserved,
                    tag,
                    size: size as u8,
                    data: 0,
                    raw: data[..3 + size].to_vec(),
                },
                3 + size,
            ));
        }
        
        // Short item format
        let size = match prefix & 0x03 {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 4,
            _ => unreachable!(),
        };
        
        if data.len() < 1 + size {
            return None;
        }
        
        let item_type = match (prefix >> 2) & 0x03 {
            0 => ItemType::Main,
            1 => ItemType::Global,
            2 => ItemType::Local,
            _ => ItemType::Reserved,
        };
        
        let tag = (prefix >> 4) & 0x0F;
        
        let value = match size {
            0 => 0,
            1 => data[1] as i8 as i32,
            2 => i16::from_le_bytes([data[1], data[2]]) as i32,
            4 => i32::from_le_bytes([data[1], data[2], data[3], data[4]]),
            _ => 0,
        };
        
        Some((
            Self {
                item_type,
                tag,
                size: size as u8,
                data: value,
                raw: data[..1 + size].to_vec(),
            },
            1 + size,
        ))
    }
    
    /// Get main tag
    pub fn main_tag(&self) -> Option<MainTag> {
        if self.item_type != ItemType::Main {
            return None;
        }
        match self.tag {
            0x08 => Some(MainTag::Input),
            0x09 => Some(MainTag::Output),
            0x0B => Some(MainTag::Feature),
            0x0A => Some(MainTag::Collection),
            0x0C => Some(MainTag::EndCollection),
            _ => None,
        }
    }
    
    /// Get global tag
    pub fn global_tag(&self) -> Option<GlobalTag> {
        if self.item_type != ItemType::Global {
            return None;
        }
        match self.tag {
            0x00 => Some(GlobalTag::UsagePage),
            0x01 => Some(GlobalTag::LogicalMinimum),
            0x02 => Some(GlobalTag::LogicalMaximum),
            0x03 => Some(GlobalTag::PhysicalMinimum),
            0x04 => Some(GlobalTag::PhysicalMaximum),
            0x05 => Some(GlobalTag::UnitExponent),
            0x06 => Some(GlobalTag::Unit),
            0x07 => Some(GlobalTag::ReportSize),
            0x08 => Some(GlobalTag::ReportId),
            0x09 => Some(GlobalTag::ReportCount),
            0x0A => Some(GlobalTag::Push),
            0x0B => Some(GlobalTag::Pop),
            _ => None,
        }
    }
    
    /// Get local tag
    pub fn local_tag(&self) -> Option<LocalTag> {
        if self.item_type != ItemType::Local {
            return None;
        }
        match self.tag {
            0x00 => Some(LocalTag::Usage),
            0x01 => Some(LocalTag::UsageMinimum),
            0x02 => Some(LocalTag::UsageMaximum),
            0x03 => Some(LocalTag::DesignatorIndex),
            0x04 => Some(LocalTag::DesignatorMinimum),
            0x05 => Some(LocalTag::DesignatorMaximum),
            0x07 => Some(LocalTag::StringIndex),
            0x08 => Some(LocalTag::StringMinimum),
            0x09 => Some(LocalTag::StringMaximum),
            0x0A => Some(LocalTag::Delimiter),
            _ => None,
        }
    }
}

/// Global state during parsing
#[derive(Debug, Clone, Default)]
pub struct GlobalState {
    pub usage_page: u16,
    pub logical_minimum: i32,
    pub logical_maximum: i32,
    pub physical_minimum: i32,
    pub physical_maximum: i32,
    pub unit_exponent: i32,
    pub unit: u32,
    pub report_size: u32,
    pub report_id: u8,
    pub report_count: u32,
}

/// Local state during parsing
#[derive(Debug, Clone, Default)]
pub struct LocalState {
    pub usages: Vec<u32>,
    pub usage_minimum: u32,
    pub usage_maximum: u32,
    pub designator_index: u32,
    pub designator_minimum: u32,
    pub designator_maximum: u32,
    pub string_index: u32,
    pub string_minimum: u32,
    pub string_maximum: u32,
}

impl LocalState {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// Report field flags
#[derive(Debug, Clone, Copy, Default)]
pub struct FieldFlags {
    /// Data (0) or Constant (1)
    pub constant: bool,
    /// Array (0) or Variable (1)
    pub variable: bool,
    /// Absolute (0) or Relative (1)
    pub relative: bool,
    /// No Wrap (0) or Wrap (1)
    pub wrap: bool,
    /// Linear (0) or Non-Linear (1)
    pub non_linear: bool,
    /// Preferred State (0) or No Preferred (1)
    pub no_preferred: bool,
    /// No Null Position (0) or Null State (1)
    pub null_state: bool,
    /// Non-Volatile (0) or Volatile (1)
    pub volatile: bool,
    /// Bit Field (0) or Buffered Bytes (1)
    pub buffered_bytes: bool,
}

impl FieldFlags {
    pub fn from_bits(bits: u32) -> Self {
        Self {
            constant: (bits & 0x001) != 0,
            variable: (bits & 0x002) != 0,
            relative: (bits & 0x004) != 0,
            wrap: (bits & 0x008) != 0,
            non_linear: (bits & 0x010) != 0,
            no_preferred: (bits & 0x020) != 0,
            null_state: (bits & 0x040) != 0,
            volatile: (bits & 0x080) != 0,
            buffered_bytes: (bits & 0x100) != 0,
        }
    }
}

/// A field in a HID report
#[derive(Debug, Clone)]
pub struct ReportField {
    /// Field type (Input, Output, Feature)
    pub field_type: MainTag,
    /// Report ID
    pub report_id: u8,
    /// Usage page
    pub usage_page: u16,
    /// Usages for this field
    pub usages: Vec<u32>,
    /// Flags
    pub flags: FieldFlags,
    /// Logical minimum
    pub logical_minimum: i32,
    /// Logical maximum
    pub logical_maximum: i32,
    /// Physical minimum
    pub physical_minimum: i32,
    /// Physical maximum
    pub physical_maximum: i32,
    /// Size in bits per field
    pub report_size: u32,
    /// Number of fields
    pub report_count: u32,
    /// Bit offset in report
    pub bit_offset: u32,
}

impl ReportField {
    /// Total size in bits
    pub fn total_bits(&self) -> u32 {
        self.report_size * self.report_count
    }
    
    /// Get usage name
    pub fn usage_name(&self, index: usize) -> String {
        if index >= self.usages.len() {
            return "Unknown".to_string();
        }
        
        let usage = self.usages[index];
        let page = (usage >> 16) as u16;
        let id = (usage & 0xFFFF) as u16;
        let page = if page == 0 { self.usage_page } else { page };
        
        match page {
            usage_page::GENERIC_DESKTOP => usage_desktop::name(id).to_string(),
            usage_page::BUTTON => format!("Button {}", id),
            _ => format!("0x{:04X}:0x{:04X}", page, id),
        }
    }
}

/// A complete HID report descriptor
#[derive(Debug, Clone)]
pub struct ReportDescriptor {
    /// Raw descriptor bytes
    pub raw: Vec<u8>,
    /// Parsed items
    pub items: Vec<HidItem>,
    /// Input report fields
    pub input_fields: Vec<ReportField>,
    /// Output report fields
    pub output_fields: Vec<ReportField>,
    /// Feature report fields
    pub feature_fields: Vec<ReportField>,
    /// Report IDs used
    pub report_ids: Vec<u8>,
    /// Collections
    pub collections: Vec<Collection>,
}

/// A HID collection
#[derive(Debug, Clone)]
pub struct Collection {
    /// Collection type
    pub collection_type: CollectionType,
    /// Usage page
    pub usage_page: u16,
    /// Usage
    pub usage: u16,
    /// Depth in collection hierarchy
    pub depth: usize,
}

impl ReportDescriptor {
    /// Parse a HID report descriptor
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        let mut items = Vec::new();
        let mut offset = 0;
        
        while offset < data.len() {
            if let Some((item, size)) = HidItem::parse(&data[offset..]) {
                items.push(item);
                offset += size;
            } else {
                break;
            }
        }
        
        let mut global = GlobalState::default();
        let mut local = LocalState::default();
        let mut global_stack: Vec<GlobalState> = Vec::new();
        
        let mut input_fields = Vec::new();
        let mut output_fields = Vec::new();
        let mut feature_fields = Vec::new();
        let mut report_ids = Vec::new();
        let mut collections = Vec::new();
        let mut collection_depth = 0;
        
        let mut bit_offset_input: HashMap<u8, u32> = HashMap::new();
        let mut bit_offset_output: HashMap<u8, u32> = HashMap::new();
        let mut bit_offset_feature: HashMap<u8, u32> = HashMap::new();
        
        for item in &items {
            match item.item_type {
                ItemType::Global => {
                    if let Some(tag) = item.global_tag() {
                        match tag {
                            GlobalTag::UsagePage => global.usage_page = item.data as u16,
                            GlobalTag::LogicalMinimum => global.logical_minimum = item.data,
                            GlobalTag::LogicalMaximum => global.logical_maximum = item.data,
                            GlobalTag::PhysicalMinimum => global.physical_minimum = item.data,
                            GlobalTag::PhysicalMaximum => global.physical_maximum = item.data,
                            GlobalTag::UnitExponent => global.unit_exponent = item.data,
                            GlobalTag::Unit => global.unit = item.data as u32,
                            GlobalTag::ReportSize => global.report_size = item.data as u32,
                            GlobalTag::ReportId => {
                                global.report_id = item.data as u8;
                                if !report_ids.contains(&global.report_id) {
                                    report_ids.push(global.report_id);
                                }
                            }
                            GlobalTag::ReportCount => global.report_count = item.data as u32,
                            GlobalTag::Push => global_stack.push(global.clone()),
                            GlobalTag::Pop => {
                                if let Some(state) = global_stack.pop() {
                                    global = state;
                                }
                            }
                        }
                    }
                }
                ItemType::Local => {
                    if let Some(tag) = item.local_tag() {
                        match tag {
                            LocalTag::Usage => {
                                let usage = item.data as u32;
                                local.usages.push(usage);
                            }
                            LocalTag::UsageMinimum => local.usage_minimum = item.data as u32,
                            LocalTag::UsageMaximum => local.usage_maximum = item.data as u32,
                            LocalTag::DesignatorIndex => local.designator_index = item.data as u32,
                            LocalTag::DesignatorMinimum => local.designator_minimum = item.data as u32,
                            LocalTag::DesignatorMaximum => local.designator_maximum = item.data as u32,
                            LocalTag::StringIndex => local.string_index = item.data as u32,
                            LocalTag::StringMinimum => local.string_minimum = item.data as u32,
                            LocalTag::StringMaximum => local.string_maximum = item.data as u32,
                            LocalTag::Delimiter => {}
                        }
                    }
                }
                ItemType::Main => {
                    if let Some(tag) = item.main_tag() {
                        match tag {
                            MainTag::Collection => {
                                let usage = local.usages.first().copied().unwrap_or(0);
                                collections.push(Collection {
                                    collection_type: CollectionType::from_byte(item.data as u8),
                                    usage_page: global.usage_page,
                                    usage: usage as u16,
                                    depth: collection_depth,
                                });
                                collection_depth += 1;
                                local.clear();
                            }
                            MainTag::EndCollection => {
                                collection_depth = collection_depth.saturating_sub(1);
                            }
                            MainTag::Input | MainTag::Output | MainTag::Feature => {
                                // Build usages list
                                let mut usages = local.usages.clone();
                                if usages.is_empty() && local.usage_minimum <= local.usage_maximum {
                                    for u in local.usage_minimum..=local.usage_maximum {
                                        usages.push(u);
                                    }
                                }
                                
                                let (bit_offsets, fields) = match tag {
                                    MainTag::Input => (&mut bit_offset_input, &mut input_fields),
                                    MainTag::Output => (&mut bit_offset_output, &mut output_fields),
                                    MainTag::Feature => (&mut bit_offset_feature, &mut feature_fields),
                                    _ => unreachable!(),
                                };
                                
                                let offset = *bit_offsets.get(&global.report_id).unwrap_or(&0);
                                
                                fields.push(ReportField {
                                    field_type: tag,
                                    report_id: global.report_id,
                                    usage_page: global.usage_page,
                                    usages,
                                    flags: FieldFlags::from_bits(item.data as u32),
                                    logical_minimum: global.logical_minimum,
                                    logical_maximum: global.logical_maximum,
                                    physical_minimum: global.physical_minimum,
                                    physical_maximum: global.physical_maximum,
                                    report_size: global.report_size,
                                    report_count: global.report_count,
                                    bit_offset: offset,
                                });
                                
                                bit_offsets.insert(
                                    global.report_id,
                                    offset + global.report_size * global.report_count,
                                );
                                
                                local.clear();
                            }
                        }
                    }
                }
                ItemType::Reserved => {}
            }
        }
        
        Ok(Self {
            raw: data.to_vec(),
            items,
            input_fields,
            output_fields,
            feature_fields,
            report_ids,
            collections,
        })
    }
    
    /// Get total input report size in bytes (excluding report ID)
    pub fn input_report_size(&self, report_id: u8) -> usize {
        let bits: u32 = self.input_fields.iter()
            .filter(|f| f.report_id == report_id)
            .map(|f| f.total_bits())
            .sum();
        ((bits + 7) / 8) as usize
    }
    
    /// Get total output report size in bytes
    pub fn output_report_size(&self, report_id: u8) -> usize {
        let bits: u32 = self.output_fields.iter()
            .filter(|f| f.report_id == report_id)
            .map(|f| f.total_bits())
            .sum();
        ((bits + 7) / 8) as usize
    }
    
    /// Get device type from collections
    pub fn device_type(&self) -> &'static str {
        for col in &self.collections {
            if col.collection_type == CollectionType::Application {
                match (col.usage_page, col.usage) {
                    (usage_page::GENERIC_DESKTOP, usage_desktop::MOUSE) => return "Mouse",
                    (usage_page::GENERIC_DESKTOP, usage_desktop::KEYBOARD) => return "Keyboard",
                    (usage_page::GENERIC_DESKTOP, usage_desktop::KEYPAD) => return "Keypad",
                    (usage_page::GENERIC_DESKTOP, usage_desktop::JOYSTICK) => return "Joystick",
                    (usage_page::GENERIC_DESKTOP, usage_desktop::GAMEPAD) => return "Gamepad",
                    (usage_page::CONSUMER, _) => return "Consumer Control",
                    (usage_page::DIGITIZER, _) => return "Digitizer",
                    (usage_page::FIDO, _) => return "FIDO Security Key",
                    _ => {}
                }
            }
        }
        "Unknown HID Device"
    }
    
    /// Print a human-readable summary
    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("Device Type: {}\n", self.device_type()));
        s.push_str(&format!("Report IDs: {:?}\n", self.report_ids));
        s.push_str(&format!("Collections: {}\n", self.collections.len()));
        s.push_str(&format!("Input Fields: {}\n", self.input_fields.len()));
        s.push_str(&format!("Output Fields: {}\n", self.output_fields.len()));
        s.push_str(&format!("Feature Fields: {}\n", self.feature_fields.len()));
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_type() {
        assert_eq!(CollectionType::from_byte(0x01), CollectionType::Application);
        assert_eq!(CollectionType::Application.name(), "Application");
    }

    #[test]
    fn test_usage_page_name() {
        assert_eq!(usage_page::name(usage_page::GENERIC_DESKTOP), "Generic Desktop");
        assert_eq!(usage_page::name(usage_page::KEYBOARD), "Keyboard/Keypad");
    }

    #[test]
    fn test_field_flags() {
        let flags = FieldFlags::from_bits(0x02);
        assert!(flags.variable);
        assert!(!flags.constant);
        assert!(!flags.relative);
    }

    // Simple mouse descriptor
    #[test]
    fn test_parse_simple() {
        let mouse_desc = [
            0x05, 0x01,  // Usage Page (Generic Desktop)
            0x09, 0x02,  // Usage (Mouse)
            0xA1, 0x01,  // Collection (Application)
            0x09, 0x01,  //   Usage (Pointer)
            0xA1, 0x00,  //   Collection (Physical)
            0x05, 0x09,  //     Usage Page (Button)
            0x19, 0x01,  //     Usage Minimum (1)
            0x29, 0x03,  //     Usage Maximum (3)
            0x15, 0x00,  //     Logical Minimum (0)
            0x25, 0x01,  //     Logical Maximum (1)
            0x95, 0x03,  //     Report Count (3)
            0x75, 0x01,  //     Report Size (1)
            0x81, 0x02,  //     Input (Variable)
            0x95, 0x01,  //     Report Count (1)
            0x75, 0x05,  //     Report Size (5)
            0x81, 0x01,  //     Input (Constant)
            0x05, 0x01,  //     Usage Page (Generic Desktop)
            0x09, 0x30,  //     Usage (X)
            0x09, 0x31,  //     Usage (Y)
            0x15, 0x81,  //     Logical Minimum (-127)
            0x25, 0x7F,  //     Logical Maximum (127)
            0x75, 0x08,  //     Report Size (8)
            0x95, 0x02,  //     Report Count (2)
            0x81, 0x06,  //     Input (Variable, Relative)
            0xC0,        //   End Collection
            0xC0,        // End Collection
        ];
        
        let desc = ReportDescriptor::parse(&mouse_desc).unwrap();
        assert_eq!(desc.device_type(), "Mouse");
        assert_eq!(desc.collections.len(), 2);
        assert_eq!(desc.input_fields.len(), 3);
    }
}
