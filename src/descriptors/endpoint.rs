//! Endpoint Descriptor Parsing
//!
//! USB Endpoint descriptors describe specific endpoints within an interface,
//! including their type (bulk, interrupt, isochronous, control), direction,
//! and transfer characteristics.

use serde::{Deserialize, Serialize};

/// USB Endpoint Descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointDescriptor {
    /// Endpoint address (includes direction in bit 7)
    pub address: u8,
    /// Endpoint number (0-15)
    pub number: u8,
    /// Endpoint direction
    pub direction: EndpointDirection,
    /// Transfer type
    pub transfer_type: TransferType,
    /// Synchronization type (for isochronous endpoints)
    pub sync_type: SyncType,
    /// Usage type (for isochronous endpoints)
    pub usage_type: UsageType,
    /// Maximum packet size
    pub max_packet_size: u16,
    /// Additional transactions per microframe (USB 2.0 high-bandwidth)
    pub additional_transactions: u8,
    /// Polling interval (interpretation depends on speed and transfer type)
    pub interval: u8,
    /// Calculated interval in milliseconds
    pub interval_ms: f64,
    /// SuperSpeed companion descriptor (if available)
    pub ss_companion: Option<SuperSpeedCompanion>,
    /// Raw extra bytes
    pub extra: Vec<u8>,
}

/// Endpoint direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndpointDirection {
    /// Host to device (OUT)
    Out,
    /// Device to host (IN)
    In,
}

impl EndpointDirection {
    /// Parse from endpoint address
    pub fn from_address(address: u8) -> Self {
        if (address & 0x80) != 0 {
            EndpointDirection::In
        } else {
            EndpointDirection::Out
        }
    }
}

/// USB Transfer Type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferType {
    /// Control transfers (endpoint 0)
    Control,
    /// Isochronous transfers (guaranteed bandwidth, no error correction)
    Isochronous,
    /// Bulk transfers (large data, error correction, no guaranteed bandwidth)
    Bulk,
    /// Interrupt transfers (small data, guaranteed latency)
    Interrupt,
}

impl TransferType {
    /// Parse from bmAttributes
    pub fn from_attributes(attrs: u8) -> Self {
        match attrs & 0x03 {
            0x00 => TransferType::Control,
            0x01 => TransferType::Isochronous,
            0x02 => TransferType::Bulk,
            0x03 => TransferType::Interrupt,
            _ => unreachable!(),
        }
    }
    
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            TransferType::Control => "Control",
            TransferType::Isochronous => "Isochronous",
            TransferType::Bulk => "Bulk",
            TransferType::Interrupt => "Interrupt",
        }
    }
}

/// Synchronization type for isochronous endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncType {
    /// No synchronization
    None,
    /// Asynchronous
    Asynchronous,
    /// Adaptive
    Adaptive,
    /// Synchronous
    Synchronous,
}

impl SyncType {
    /// Parse from bmAttributes
    pub fn from_attributes(attrs: u8) -> Self {
        match (attrs >> 2) & 0x03 {
            0x00 => SyncType::None,
            0x01 => SyncType::Asynchronous,
            0x02 => SyncType::Adaptive,
            0x03 => SyncType::Synchronous,
            _ => unreachable!(),
        }
    }
}

/// Usage type for isochronous endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsageType {
    /// Data endpoint
    Data,
    /// Feedback endpoint
    Feedback,
    /// Implicit feedback data endpoint
    ImplicitFeedback,
    /// Reserved
    Reserved,
}

impl UsageType {
    /// Parse from bmAttributes
    pub fn from_attributes(attrs: u8) -> Self {
        match (attrs >> 4) & 0x03 {
            0x00 => UsageType::Data,
            0x01 => UsageType::Feedback,
            0x02 => UsageType::ImplicitFeedback,
            0x03 => UsageType::Reserved,
            _ => unreachable!(),
        }
    }
}

/// SuperSpeed Endpoint Companion Descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperSpeedCompanion {
    /// Maximum number of packets the endpoint can send/receive in a service interval
    pub max_burst: u8,
    /// Attributes (depends on endpoint type)
    pub attributes: u8,
    /// Number of bytes per service interval (isochronous only)
    pub bytes_per_interval: u16,
    /// For bulk endpoints: max streams supported (2^n)
    pub max_streams: Option<u8>,
    /// For isochronous endpoints: mult value
    pub mult: Option<u8>,
    /// SuperSpeedPlus Isochronous Companion (if present)
    pub ssp_isoc_companion: Option<SuperSpeedPlusIsocCompanion>,
}

/// SuperSpeed Plus Isochronous Endpoint Companion Descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperSpeedPlusIsocCompanion {
    /// Reserved (must be zero)
    pub reserved: u16,
    /// Bytes per interval (actual value, not encoded)
    pub bytes_per_interval: u32,
}

/// Parse an endpoint descriptor from rusb
pub fn parse_endpoint(endpoint: &rusb::EndpointDescriptor) -> EndpointDescriptor {
    let address = endpoint.address();
    let _attributes = endpoint.transfer_type() as u8
        | ((endpoint.sync_type() as u8) << 2)
        | ((endpoint.usage_type() as u8) << 4);
    
    let transfer_type = match endpoint.transfer_type() {
        rusb::TransferType::Control => TransferType::Control,
        rusb::TransferType::Isochronous => TransferType::Isochronous,
        rusb::TransferType::Bulk => TransferType::Bulk,
        rusb::TransferType::Interrupt => TransferType::Interrupt,
    };
    
    let sync_type = match endpoint.sync_type() {
        rusb::SyncType::NoSync => SyncType::None,
        rusb::SyncType::Asynchronous => SyncType::Asynchronous,
        rusb::SyncType::Adaptive => SyncType::Adaptive,
        rusb::SyncType::Synchronous => SyncType::Synchronous,
    };
    
    let usage_type = match endpoint.usage_type() {
        rusb::UsageType::Data => UsageType::Data,
        rusb::UsageType::Feedback => UsageType::Feedback,
        rusb::UsageType::FeedbackData => UsageType::ImplicitFeedback,
        rusb::UsageType::Reserved => UsageType::Reserved,
    };
    
    // Parse max packet size and additional transactions (high-bandwidth)
    let raw_max_packet = endpoint.max_packet_size();
    let max_packet_size = raw_max_packet & 0x07FF;
    let additional_transactions = ((raw_max_packet >> 11) & 0x03) as u8;
    
    // Calculate interval in milliseconds
    let interval = endpoint.interval();
    let interval_ms = calculate_interval_ms(interval, transfer_type);
    
    // Try to parse SuperSpeed companion from extra bytes
    let extra_bytes = endpoint.extra().unwrap_or(&[]);
    let ss_companion = parse_ss_companion(extra_bytes);
    
    EndpointDescriptor {
        address,
        number: address & 0x0F,
        direction: EndpointDirection::from_address(address),
        transfer_type,
        sync_type,
        usage_type,
        max_packet_size,
        additional_transactions,
        interval,
        interval_ms,
        ss_companion,
        extra: extra_bytes.to_vec(),
    }
}

/// Calculate interval in milliseconds based on transfer type
fn calculate_interval_ms(interval: u8, transfer_type: TransferType) -> f64 {
    match transfer_type {
        TransferType::Interrupt | TransferType::Isochronous => {
            // For high-speed, interval is 2^(interval-1) microframes (125µs each)
            // For full/low speed, interval is in milliseconds
            // We assume high-speed here; platform enrichment can adjust
            if interval > 0 {
                let microframes = 1u32 << (interval - 1);
                (microframes as f64) * 0.125
            } else {
                0.0
            }
        }
        TransferType::Bulk | TransferType::Control => 0.0,
    }
}

/// Parse SuperSpeed companion descriptor from extra bytes
fn parse_ss_companion(extra: &[u8]) -> Option<SuperSpeedCompanion> {
    if extra.len() < 6 {
        return None;
    }
    
    // Look for SuperSpeed Endpoint Companion descriptor (type 0x30)
    let mut offset = 0;
    while offset + 2 < extra.len() {
        let length = extra[offset] as usize;
        let desc_type = extra[offset + 1];
        
        if length < 2 || offset + length > extra.len() {
            break;
        }
        
        if desc_type == 0x30 && length >= 6 {
            let max_burst = extra[offset + 2];
            let attributes = extra[offset + 3];
            let bytes_per_interval = u16::from_le_bytes([extra[offset + 4], extra[offset + 5]]);
            
            // For bulk endpoints, attributes contains max streams
            let max_streams = if (attributes & 0x1F) > 0 {
                Some(attributes & 0x1F)
            } else {
                None
            };
            
            // For isochronous endpoints, attributes contains mult
            let mult = if (attributes & 0x03) > 0 {
                Some(attributes & 0x03)
            } else {
                None
            };
            
            // Look for SuperSpeedPlus Isochronous Companion (type 0x31)
            let ssp_isoc_companion = parse_ssp_isoc_companion(&extra[offset + length..]);
            
            return Some(SuperSpeedCompanion {
                max_burst,
                attributes,
                bytes_per_interval,
                max_streams,
                mult,
                ssp_isoc_companion,
            });
        }
        
        offset += length;
    }
    
    None
}

/// Parse SuperSpeedPlus Isochronous Companion descriptor
fn parse_ssp_isoc_companion(extra: &[u8]) -> Option<SuperSpeedPlusIsocCompanion> {
    if extra.len() < 8 {
        return None;
    }
    
    let mut offset = 0;
    while offset + 2 < extra.len() {
        let length = extra[offset] as usize;
        let desc_type = extra[offset + 1];
        
        if length < 2 || offset + length > extra.len() {
            break;
        }
        
        if desc_type == 0x31 && length >= 8 {
            let reserved = u16::from_le_bytes([extra[offset + 2], extra[offset + 3]]);
            let bytes_per_interval = u32::from_le_bytes([
                extra[offset + 4],
                extra[offset + 5],
                extra[offset + 6],
                extra[offset + 7],
            ]);
            
            return Some(SuperSpeedPlusIsocCompanion {
                reserved,
                bytes_per_interval,
            });
        }
        
        offset += length;
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_direction() {
        assert_eq!(EndpointDirection::from_address(0x01), EndpointDirection::Out);
        assert_eq!(EndpointDirection::from_address(0x81), EndpointDirection::In);
        assert_eq!(EndpointDirection::from_address(0x82), EndpointDirection::In);
        assert_eq!(EndpointDirection::from_address(0x02), EndpointDirection::Out);
    }

    #[test]
    fn test_transfer_type() {
        assert_eq!(TransferType::from_attributes(0x00), TransferType::Control);
        assert_eq!(TransferType::from_attributes(0x01), TransferType::Isochronous);
        assert_eq!(TransferType::from_attributes(0x02), TransferType::Bulk);
        assert_eq!(TransferType::from_attributes(0x03), TransferType::Interrupt);
    }

    #[test]
    fn test_interval_calculation() {
        // Interrupt endpoint with interval=4 (2^3 = 8 microframes = 1ms)
        let ms = calculate_interval_ms(4, TransferType::Interrupt);
        assert!((ms - 1.0).abs() < 0.001);
        
        // Bulk endpoints have no interval
        let ms_bulk = calculate_interval_ms(4, TransferType::Bulk);
        assert_eq!(ms_bulk, 0.0);
    }
}
