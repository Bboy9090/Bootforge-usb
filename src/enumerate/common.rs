use crate::{api::UsbEnumerator, errors::UsbError, model::*};

#[derive(Default)]
pub struct FallbackEnumerator;

impl UsbEnumerator for FallbackEnumerator {
    fn enumerate(&self) -> Result<Vec<UsbDeviceRecord>, UsbError> {
        // Fallback implementation using rusb for basic enumeration
        let devices = rusb::devices()?;
        let mut records = Vec::new();

        for device in devices.iter() {
            let device_desc = match device.device_descriptor() {
                Ok(desc) => desc,
                Err(_) => continue,
            };

            let handle = device.open();
            let (manufacturer, product, serial_number) = if let Ok(h) = &handle {
                let timeout = std::time::Duration::from_secs(1);
                let languages = match h.read_languages(timeout) {
                    Ok(langs) if !langs.is_empty() => langs,
                    _ => continue,
                };
                let lang = languages[0];

                let manufacturer = device_desc
                    .manufacturer_string_index()
                    .and_then(|idx| h.read_string_descriptor(lang, idx, timeout).ok());
                let product = device_desc
                    .product_string_index()
                    .and_then(|idx| h.read_string_descriptor(lang, idx, timeout).ok());
                let serial = device_desc
                    .serial_number_string_index()
                    .and_then(|idx| h.read_string_descriptor(lang, idx, timeout).ok());

                (manufacturer, product, serial)
            } else {
                (None, None, None)
            };

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
