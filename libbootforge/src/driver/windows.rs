//! Passive Windows USB driver enrichment using SetupAPI and CfgMgr32.
//!
//! This backend only reads present-device metadata. It never installs, updates, enables,
//! disables, binds, removes, or otherwise changes a driver or device node.

use super::{
    DriverBackend, DriverConfidence, DriverEvidence, DriverInspector, DriverReport, DriverState,
    WindowsDriverInspector,
};
use crate::types::DeviceInfo;
use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};

const DIGCF_PRESENT: u32 = 0x0000_0002;
const DIGCF_ALLCLASSES: u32 = 0x0000_0004;
const SPDRP_SERVICE: u32 = 0x0000_0004;
const SPDRP_MFG: u32 = 0x0000_000B;
const SPDRP_DRIVER: u32 = 0x0000_0009;
const ERROR_NO_MORE_ITEMS: u32 = 259;
const CR_SUCCESS: u32 = 0;
const CM_PROB_DISABLED: u32 = 0x0000_0016;
const INVALID_HANDLE_VALUE: isize = -1;

#[repr(C)]
#[derive(Clone, Copy)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[repr(C)]
struct SpDevInfoData {
    cb_size: u32,
    class_guid: Guid,
    dev_inst: u32,
    reserved: usize,
}

#[link(name = "setupapi")]
extern "system" {
    fn SetupDiGetClassDevsW(
        class_guid: *const Guid,
        enumerator: *const u16,
        hwnd_parent: isize,
        flags: u32,
    ) -> isize;
    fn SetupDiEnumDeviceInfo(
        device_info_set: isize,
        member_index: u32,
        device_info_data: *mut SpDevInfoData,
    ) -> i32;
    fn SetupDiGetDeviceInstanceIdW(
        device_info_set: isize,
        device_info_data: *mut SpDevInfoData,
        device_instance_id: *mut u16,
        device_instance_id_size: u32,
        required_size: *mut u32,
    ) -> i32;
    fn SetupDiGetDeviceRegistryPropertyW(
        device_info_set: isize,
        device_info_data: *mut SpDevInfoData,
        property: u32,
        property_reg_data_type: *mut u32,
        property_buffer: *mut u8,
        property_buffer_size: u32,
        required_size: *mut u32,
    ) -> i32;
    fn SetupDiDestroyDeviceInfoList(device_info_set: isize) -> i32;
}

#[link(name = "cfgmgr32")]
extern "system" {
    fn CM_Get_DevNode_Status(
        status: *mut u32,
        problem_number: *mut u32,
        dev_inst: u32,
        flags: u32,
    ) -> u32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetLastError() -> u32;
}

impl DriverInspector for WindowsDriverInspector {
    fn backend(&self) -> DriverBackend {
        DriverBackend::WindowsSetupApi
    }

    fn inspect(&self, device: &DeviceInfo) -> crate::Result<DriverReport> {
        unsafe { inspect_present_device(device) }
    }
}

unsafe fn inspect_present_device(device: &DeviceInfo) -> crate::Result<DriverReport> {
    let set = SetupDiGetClassDevsW(null(), null(), 0, DIGCF_PRESENT | DIGCF_ALLCLASSES);

    if set == INVALID_HANDLE_VALUE {
        return Ok(report_error(
            DriverState::PermissionDenied,
            format!(
                "SetupDiGetClassDevsW failed with Win32 error {}",
                GetLastError()
            ),
        ));
    }

    let target = format!("VID_{:04X}&PID_{:04X}", device.vendor_id, device.product_id);
    let serial = device
        .serial_number
        .as_deref()
        .map(|value| value.trim().to_ascii_uppercase())
        .filter(|value| !value.is_empty());

    let mut index = 0_u32;
    let result = loop {
        let mut info: SpDevInfoData = zeroed();
        info.cb_size = size_of::<SpDevInfoData>() as u32;

        if SetupDiEnumDeviceInfo(set, index, &mut info) == 0 {
            let error = GetLastError();
            break if error == ERROR_NO_MORE_ITEMS {
                missing_report(device)
            } else {
                report_error(
                    DriverState::Failed,
                    format!("SetupDiEnumDeviceInfo failed with Win32 error {error}"),
                )
            };
        }
        index = index.saturating_add(1);

        let Some(instance_id) = read_instance_id(set, &mut info) else {
            continue;
        };
        let normalized_id = instance_id.to_ascii_uppercase();
        if !normalized_id.contains(&target) {
            continue;
        }
        if let Some(expected_serial) = &serial {
            if !normalized_id.contains(expected_serial) {
                continue;
            }
        }

        let service = read_property_string(set, &mut info, SPDRP_SERVICE);
        let manufacturer = read_property_string(set, &mut info, SPDRP_MFG);
        let driver_key = read_property_string(set, &mut info, SPDRP_DRIVER);
        let mut status = 0_u32;
        let mut problem = 0_u32;
        let cfg_result = CM_Get_DevNode_Status(&mut status, &mut problem, info.dev_inst, 0);

        let mut evidence = vec![DriverEvidence::BackendRecord, DriverEvidence::DeviceNode];
        if service.is_some() {
            evidence.push(DriverEvidence::ServiceName);
            evidence.push(DriverEvidence::KernelBinding);
        }
        if manufacturer.is_some() {
            evidence.push(DriverEvidence::Provider);
        }
        if driver_key.is_some() {
            evidence.push(DriverEvidence::Version);
        }
        if cfg_result == CR_SUCCESS && problem != 0 {
            evidence.push(DriverEvidence::ProblemCode);
        }

        let state = if cfg_result == CR_SUCCESS && problem == CM_PROB_DISABLED {
            DriverState::Disabled
        } else if cfg_result == CR_SUCCESS && problem != 0 {
            DriverState::Failed
        } else if service.is_some() {
            DriverState::Bound
        } else {
            DriverState::Present
        };

        break DriverReport {
            backend: DriverBackend::WindowsSetupApi,
            state,
            confidence: if serial.is_some() {
                DriverConfidence::Exact
            } else {
                DriverConfidence::High
            },
            driver_name: service.clone(),
            service_name: service,
            provider: manufacturer,
            version: driver_key,
            signed: None,
            problem_code: if cfg_result == CR_SUCCESS && problem != 0 {
                Some(format!("CM_PROB_{problem:#010x}"))
            } else {
                None
            },
            device_node: Some(instance_id),
            evidence,
            message: Some(
                "present Windows device matched through SetupAPI; CfgMgr32 status queried read-only"
                    .into(),
            ),
        };
    };

    SetupDiDestroyDeviceInfoList(set);
    Ok(result)
}

unsafe fn read_instance_id(set: isize, info: &mut SpDevInfoData) -> Option<String> {
    let mut required = 0_u32;
    SetupDiGetDeviceInstanceIdW(set, info, null_mut(), 0, &mut required);
    if required == 0 {
        return None;
    }
    let mut buffer = vec![0_u16; required as usize];
    if SetupDiGetDeviceInstanceIdW(set, info, buffer.as_mut_ptr(), required, &mut required) == 0 {
        return None;
    }
    Some(wide_to_string(&buffer))
}

unsafe fn read_property_string(
    set: isize,
    info: &mut SpDevInfoData,
    property: u32,
) -> Option<String> {
    let mut required = 0_u32;
    let mut data_type = 0_u32;
    SetupDiGetDeviceRegistryPropertyW(
        set,
        info,
        property,
        &mut data_type,
        null_mut(),
        0,
        &mut required,
    );
    if required < 2 {
        return None;
    }
    let mut buffer = vec![0_u8; required as usize];
    if SetupDiGetDeviceRegistryPropertyW(
        set,
        info,
        property,
        &mut data_type,
        buffer.as_mut_ptr(),
        required,
        &mut required,
    ) == 0
    {
        return None;
    }
    let wide = std::slice::from_raw_parts(buffer.as_ptr() as *const u16, buffer.len() / 2);
    let value = wide_to_string(wide);
    (!value.is_empty()).then_some(value)
}

fn wide_to_string(buffer: &[u16]) -> String {
    let end = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..end]).trim().to_string()
}

fn missing_report(device: &DeviceInfo) -> DriverReport {
    DriverReport {
        backend: DriverBackend::WindowsSetupApi,
        state: DriverState::Missing,
        confidence: DriverConfidence::Medium,
        driver_name: None,
        service_name: None,
        provider: None,
        version: None,
        signed: None,
        problem_code: None,
        device_node: None,
        evidence: vec![DriverEvidence::BackendRecord],
        message: Some(format!(
            "no present SetupAPI node matched VID_{:04X}&PID_{:04X}",
            device.vendor_id, device.product_id
        )),
    }
}

fn report_error(state: DriverState, message: String) -> DriverReport {
    DriverReport {
        backend: DriverBackend::WindowsSetupApi,
        state,
        confidence: DriverConfidence::High,
        driver_name: None,
        service_name: None,
        provider: None,
        version: None,
        signed: None,
        problem_code: None,
        device_node: None,
        evidence: vec![DriverEvidence::PermissionError],
        message: Some(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_string_conversion_stops_at_nul() {
        let source = [b'U' as u16, b'S' as u16, b'B' as u16, 0, b'X' as u16];
        assert_eq!(wide_to_string(&source), "USB");
    }
}
