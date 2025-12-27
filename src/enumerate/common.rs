use crate::{api::UsbEnumerator, errors::UsbError, model::*};
use std::time::Duration;

const USB_DESCRIPTOR_TIMEOUT: Duration = Duration::from_secs(1);

/// FallbackEnumerator provides basic USB device enumeration using rusb/libusb.
/// 
/// This enumerator implements the UsbEnumerator trait and serves as a fallback
/// implementation that works across all platforms. It performs the core detection
/// pipeline stages:
/// - Transport scanning via libusb
/// - Descriptor reading (manufacturer, product, serial)
/// - Basic device record creation
/// 
/// For platform-specific enrichment (paths, drivers), use the `enumerate_all()`
/// function from the `enumerate` module instead.
#[derive(Default)]
pub struct FallbackEnumerator;

impl UsbEnumerator for FallbackEnumerator {
    /// Enumerate USB devices using rusb for basic cross-platform detection.
    /// 
    /// This implementation probes candidate devices and attempts to read their
    /// descriptors. It provides the foundation for device detection but does
    /// not include platform-specific enrichment.
    fn enumerate(&self) -> Result<Vec<UsbDeviceRecord>, UsbError> {
        // Use rusb to scan USB transports and probe candidates
        let devices = rusb::devices()?;
        let mut records = Vec::new();

        for device in devices.iter() {
            // Read basic device descriptor (Stage 1: Candidate probing)
            let device_desc = match device.device_descriptor() {
                Ok(desc) => desc,
                Err(_) => continue, // Skip devices we can't access
            };

            // Attempt to open device and read string descriptors (Stage 2)
            // Note: This may fail due to permissions; we handle it gracefully
            let handle = device.open();
            let (manufacturer, product, serial_number) = if let Ok(h) = &handle {
                let languages = match h.read_languages(USB_DESCRIPTOR_TIMEOUT) {
                    Ok(langs) if !langs.is_empty() => langs,
                    _ => continue,
                };
                let lang = languages[0];

                let manufacturer = device_desc
                    .manufacturer_string_index()
                    .and_then(|idx| h.read_string_descriptor(lang, idx, USB_DESCRIPTOR_TIMEOUT).ok());
                let product = device_desc
                    .product_string_index()
                    .and_then(|idx| h.read_string_descriptor(lang, idx, USB_DESCRIPTOR_TIMEOUT).ok());
                let serial = device_desc
                    .serial_number_string_index()
                    .and_then(|idx| h.read_string_descriptor(lang, idx, USB_DESCRIPTOR_TIMEOUT).ok());

                (manufacturer, product, serial)
            } else {
                (None, None, None)
            };

            // Build confirmed device record with available information
            let record = UsbDeviceRecord {
                id: UsbId::new(device_desc.vendor_id(), device_desc.product_id()),
                location: UsbLocation {
                    bus: Some(device.bus_number()),
                    address: Some(device.address()),
                    port_path: None,
                },
                descriptor: UsbDescriptorSummary {
                    manufacturer,
                    product,
                    serial_number,
                    device_class: Some(device_desc.class_code()),
                    device_subclass: Some(device_desc.sub_class_code()),
                    device_protocol: Some(device_desc.protocol_code()),
                    usb_version: Some(format!(
                        "{}.{}",
                        device_desc.usb_version().major(),
                        device_desc.usb_version().minor()
                    )),
                },
                driver: DriverStatus::Unknown,
                health: LinkHealth::Good,
                tags: Vec::new(),
                raw_data: None,
            };

            records.push(record);
        }

        Ok(records)
    }
}
