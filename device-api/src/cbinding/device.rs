#![allow(clippy::missing_safety_doc)]
use std::ffi::CString;
use std::mem;

use libc::c_char;

use crate::blocking;
use crate::{arch, cbinding, cbinding::device_handle, cbinding::err_code, device};

unsafe fn device_mut(handle: device_handle) -> &'static mut device::Device {
    handle.as_mut().expect("invalid device_handle pointer")
}

/// \brief Retrieve the name of the device (e.g., npu0).
///
/// \remark output buffer must be allocated from outside of FFI boundary,
/// and retrieved C string must be destroyed by `free_string`.
///
/// @param handle device_handle of Furiosa NPU device.
/// @param[out] output output buffer for pointer to C string.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_name(
    handle: device_handle,
    output: *mut *mut c_char,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    match CString::new(device.name()) {
        Ok(cstring) => {
            *output = cstring.into_raw();
            cbinding::error_code::ok
        }
        Err(_) => cbinding::error_code::null_error,
    }
}

/// \brief Retrieve the device index (e.g., 0 for npu0).
///
/// \remark output buffer must be allocated from outside of FFI boundary.
///
/// @param handle device_handle of Furiosa NPU device.
/// @param[out] output output buffer for index of device.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_index(
    handle: device_handle,
    output: *mut u8,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    *output = device.device_index();
    cbinding::error_code::ok
}

/// \brief Retrieve `Arch` of the device(e.g., `Warboy`).
///
/// \remark output buffer must be allocated from outside of FFI boundary.
///
/// @param handle device_handle of Furiosa NPU device.
/// @param[out] output output buffer for arch of device.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_arch(
    handle: device_handle,
    output: *mut cbinding::arch::Arch,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    match device.arch() {
        arch::Arch::WarboyA0 => *output = cbinding::arch::Arch::WarboyA0,
        arch::Arch::WarboyB0 => *output = cbinding::arch::Arch::Warboy,
        arch::Arch::Renegade => *output = cbinding::arch::Arch::Renegade,
        arch::Arch::U250 => *output = cbinding::arch::Arch::U250,
    }
    cbinding::error_code::ok
}

/// \brief Retrieve a liveness state of the device.
///
/// \remark output buffer must be allocated from outside of FFI boundary.
///
/// @param handle device_handle of Furiosa NPU device.
/// @param[out] output output buffer for liveness of device.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_liveness(
    handle: device_handle,
    output: *mut bool,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    match device.alive() {
        Ok(alive) => {
            *output = alive;
            cbinding::error_code::ok
        }
        Err(err) => err_code(err),
    }
}

/// \brief Output of `get_device_error_states`
#[repr(C)]
pub struct ErrorStatesKeyValuePair {
    pub key: *const c_char,
    pub value: u32,
}

/// \brief Retrieve error states of the device.
///
/// \remark output buffer must be allocated from outside of FFI boundary,
/// and retrieved array of ErrorStatesKeyValuePair must be destroyed by `destroy_error_states`.
///
/// @param handle device_handle of Furiosa NPU device.
/// @param[out] output output buffer for array of `ErrorStatesKeyValuePair`.
/// @param[out] output_len output buffer for length of array.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_error_states(
    handle: device_handle,
    output: *mut *mut ErrorStatesKeyValuePair,
    output_len: *mut u8,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output_len, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    match device.atr_error() {
        Ok(hashmap) => {
            let mut output_vec = Vec::new();
            for (k, v) in hashmap.iter() {
                output_vec.push(ErrorStatesKeyValuePair {
                    key: CString::new(k.clone()).unwrap().into_raw(),
                    value: *v,
                })
            }

            output_vec.shrink_to_fit();
            *output = output_vec.as_mut_ptr();
            *output_len = output_vec.len() as u8;
            mem::forget(output_vec);
            cbinding::error_code::ok
        }
        Err(err) => err_code(err),
    }
}

/// \brief Safely free device error states array of `ErrorStatesKeyValuePair` allocated by `get_device_error_states`.
///
/// @param raw pointer to array of `ErrorStatesKeyValuePair`.
/// @param len length of array.
#[no_mangle]
pub unsafe extern "C" fn destroy_error_states(raw: *mut ErrorStatesKeyValuePair, len: u8) {
    ffi_helpers::null_pointer_check!(raw);
    let vec = Vec::from_raw_parts(raw, len as usize, len as usize);
    for v in vec.iter() {
        free_string(v.key);
    }
    drop(vec);
}

/// \brief Retrieve PCI bus number of the device.
///
/// \remark output buffer must be allocated from outside of FFI boundary.
///
/// @param handle device_handle of Furiosa NPU device.
/// @param[out] output output buffer for PCI bus number of device.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_pci_bus_number(
    handle: device_handle,
    output: *mut *mut c_char,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    match device.busname() {
        Ok(busname) => match CString::new(busname) {
            Ok(cstring) => {
                *output = cstring.into_raw();
                cbinding::error_code::ok
            }
            Err(_) => cbinding::error_code::null_error,
        },
        Err(err) => err_code(err),
    }
}

/// \brief Retrieve PCI device ID of the device.
///
/// \remark output buffer must be allocated from outside of FFI boundary,
/// and retrieved C string must be destroyed by `free_string`.
///
/// @param handle device_handle of Furiosa NPU device.
/// @param[out] output output buffer for PCI bus number of device.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_pci_dev_id(
    handle: device_handle,
    output: *mut *mut c_char,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    match device.pci_dev() {
        Ok(pci_dev_id) => match CString::new(pci_dev_id) {
            Ok(cstring) => {
                *output = cstring.into_raw();
                cbinding::error_code::ok
            }
            Err(_) => cbinding::error_code::null_error,
        },
        Err(err) => err_code(err),
    }
}

/// \brief Retrieve serial number of the device.
///
/// \remark output buffer must be allocated from outside of FFI boundary,
/// and retrieved C string must be destroyed by `free_string`.
///
/// @param handle device_handle of Furiosa NPU device.
/// @param[out] output output buffer for serial number of device.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_serial_number(
    handle: device_handle,
    output: *mut *mut c_char,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    match device.device_sn() {
        Ok(serial_number) => match CString::new(serial_number) {
            Ok(cstring) => {
                *output = cstring.into_raw();
                cbinding::error_code::ok
            }
            Err(_) => cbinding::error_code::null_error,
        },
        Err(err) => err_code(err),
    }
}

/// \brief Retrieve UUID of the device.
///
/// \remark output buffer must be allocated from outside of FFI boundary,
/// and retrieved C string must be destroyed by `free_string`.
///
/// @param handle device_handle of Furiosa NPU device.
/// @param[out] output output buffer for UUID of device.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_uuid(
    handle: device_handle,
    output: *mut *mut c_char,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    match device.device_uuid() {
        Ok(uuid) => match CString::new(uuid) {
            Ok(cstring) => {
                *output = cstring.into_raw();
                cbinding::error_code::ok
            }
            Err(_) => cbinding::error_code::null_error,
        },
        Err(err) => err_code(err),
    }
}

/// \brief Retrieves firmware revision from the device.
///
/// \remark output buffer must be allocated from outside of FFI boundary,
/// and retrieved C string must be destroyed by `free_string`.
///
/// @param handle device_handle of Furiosa NPU device.
/// @param[out] output output buffer for firmware revision of device.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_firmware_version(
    handle: device_handle,
    output: *mut *mut c_char,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    match device.firmware_version() {
        Ok(firmware_version) => match CString::new(firmware_version) {
            Ok(cstring) => {
                *output = cstring.into_raw();
                cbinding::error_code::ok
            }
            Err(_) => cbinding::error_code::null_error,
        },
        Err(err) => err_code(err),
    }
}

/// \brief Retrieves driver version for the device.
///
/// \remark output buffer must be allocated from outside of FFI boundary,
/// and retrieved C string must be destroyed by `free_string`.
///
/// @param handle device_handle of Furiosa NPU device.
/// @param[out] output output buffer for driver revision of device.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_driver_version(
    handle: device_handle,
    output: *mut *mut c_char,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    match device.driver_version() {
        Ok(driver_version) => match CString::new(driver_version) {
            Ok(cstring) => {
                *output = cstring.into_raw();
                cbinding::error_code::ok
            }
            Err(_) => cbinding::error_code::null_error,
        },
        Err(err) => err_code(err),
    }
}

/// \brief Retrieves uptime of the device.
///
/// \remark output buffer must be allocated from outside of FFI boundary.
///
/// @param handle device_handle of Furiosa NPU device.
/// @param[out] output output buffer for driver revision of device.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_heartbeat(
    handle: device_handle,
    output: *mut u32,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    match device.heartbeat() {
        Ok(heartbeat) => {
            *output = heartbeat;
            cbinding::error_code::ok
        }
        Err(err) => err_code(err),
    }
}

/// \brief Retrieve NUMA node ID associated with the NPU's PCI lane
///
/// \remark output buffer must be allocated from outside of FFI boundary.
///
/// @param handle device_handle of Furiosa NPU device.
/// @param[out] output output buffer for NUMA node ID of device.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_numa_node(
    handle: device_handle,
    output: *mut u8,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    match device.numa_node() {
        Ok(numa_node) => match numa_node {
            device::NumaNode::UnSupported => cbinding::error_code::unsupported_error,
            device::NumaNode::Id(idx) => {
                *output = idx as u8;
                cbinding::error_code::ok
            }
        },
        Err(err) => err_code(err),
    }
}

/// \brief Retrieve the number of cores
///
/// \remark output buffer must be allocated from outside of FFI boundary.
///
/// @param handle device_handle of Furiosa NPU device.
/// @param[out] output output buffer for the number of cores of device.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_core_num(
    handle: device_handle,
    output: *mut u8,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    *output = device.core_num();
    cbinding::error_code::ok
}

/// \brief Retrieve the core indices
///
/// \remark output buffer must be allocated from outside of FFI boundary,
/// and retrieved array of core id must be destroyed by `destroy_device_core_ids`.
///
/// @param handle device_handle of Furiosa NPU device.
/// @param[out] output output buffer for array of core id.
/// @param[out] output_len output buffer for length of array.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_core_ids(
    handle: device_handle,
    output: *mut *mut u8,
    output_len: *mut u8,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output_len, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    let mut cores = device.cores().clone();
    cores.shrink_to_fit();
    *output = cores.as_mut_ptr();
    *output_len = cores.len() as u8;
    mem::forget(cores);
    cbinding::error_code::ok
}

/// \brief Safely free the array of device core id that is allocated by `get_device_core_ids`.
///
/// @param raw pointer to array of device core id.
/// @param len length of array.
#[no_mangle]
pub unsafe extern "C" fn destroy_device_core_ids(raw: *mut u8, len: u8) {
    ffi_helpers::null_pointer_check!(raw);
    let vec = Vec::from_raw_parts(raw, len as usize, len as usize);
    drop(vec);
}

/// \brief Represent a device mode
#[repr(C)]
pub enum DeviceMode {
    Single,
    Fusion,
    MultiCore,
}

/// \brief Output of `get_device_files`
#[repr(C)]
pub struct DeviceFile {
    pub device_index: u8,
    pub core_range: CoreRange,
    pub path: *const c_char,
    pub mode: DeviceMode,
}

pub(crate) fn transform_device_file(origin: &device::DeviceFile) -> cbinding::device::DeviceFile {
    DeviceFile {
        device_index: origin.device_index,
        core_range: match origin.core_range() {
            device::CoreRange::All => {
                CoreRange {
                    range_type: CoreRangeType::All,
                    // It would be nice, I we could fill real values here...
                    start: 0,
                    end: 0,
                }
            }
            device::CoreRange::Range(r) => CoreRange {
                range_type: CoreRangeType::Range,
                start: r.0,
                end: r.1,
            },
        },
        path: CString::new(origin.path().as_path().to_str().unwrap())
            .unwrap()
            .into_raw(),
        mode: match origin.mode() {
            device::DeviceMode::Single => DeviceMode::Single,
            device::DeviceMode::Fusion => DeviceMode::Fusion,
            device::DeviceMode::MultiCore => DeviceMode::MultiCore,
        },
    }
}

/// \brief Retrieve the list device files under the given device.
///
/// \remark output buffer must be allocated from outside of FFI boundary,
/// and retrieved array of DeviceFile must be destroyed by `destroy_device_files`.
///
/// @param handle device_handle of Furiosa NPU device.
/// @param[out] output output buffer for array of DeviceFile.
/// @param[out] output_len output buffer for length of array.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_files(
    handle: device_handle,
    output: *mut *mut DeviceFile,
    output_len: *mut u8,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output_len, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    let mut device_files_vec: Vec<DeviceFile> = device
        .dev_files()
        .iter()
        .map(transform_device_file)
        .collect();
    device_files_vec.shrink_to_fit();
    *output = device_files_vec.as_mut_ptr();
    *output_len = device_files_vec.len() as u8;
    mem::forget(device_files_vec);
    cbinding::error_code::ok
}

/// \brief Safely free the array of DeviceFile that is allocated by `get_device_files`.
///
/// @param raw pointer to array of DeviceFile.
/// @param len length of array.
#[no_mangle]
pub unsafe extern "C" fn destroy_device_files(raw: *mut DeviceFile, len: u8) {
    ffi_helpers::null_pointer_check!(raw);
    let vec = Vec::from_raw_parts(raw, len as usize, len as usize);
    for v in vec.iter() {
        free_string(v.path);
    }
    drop(vec)
}

fn transform_core_status(origin: device::CoreStatus) -> CoreStatus {
    match origin {
        device::CoreStatus::Available => CoreStatus::Available,
        device::CoreStatus::Occupied(_) => CoreStatus::Occupied,
        device::CoreStatus::Unavailable => CoreStatus::Unavailable,
    }
}

/// \brief Examine a specific core of the device, whether it is available or not.
///
/// \remark output buffer must be allocated from outside of FFI boundary.
///
/// @param handle device_handle of Furiosa NPU device.
/// @param core_idx index of a specific core.
/// @param[out] output output buffer for core status.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_core_status(
    handle: device_handle,
    core_idx: u8,
    output: *mut CoreStatus,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    match blocking::get_status_all(device) {
        Ok(hash_map) => match hash_map.get(&core_idx) {
            None => cbinding::error_code::invalid_input,
            Some(core_status) => {
                *output = transform_core_status(core_status.clone());
                cbinding::error_code::ok
            }
        },
        Err(err) => err_code(err),
    }
}

/// \brief Retrieve the file descriptor occupied a specific core.
///
/// \remark output buffer must be allocated from outside of FFI boundary,
/// and retrieved C string must be destroyed by `free_string`
///
/// @param handle device_handle of Furiosa NPU device.
/// @param core_idx index of a specific core.
/// @param[out] output output buffer for file descriptor.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_core_occupied_fd(
    handle: device_handle,
    core_idx: u8,
    output: *mut *mut c_char,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    match blocking::get_status_all(device) {
        Ok(hash_map) => match hash_map.get(&core_idx) {
            None => cbinding::error_code::invalid_input,
            Some(core_status) => match core_status.clone() {
                device::CoreStatus::Occupied(fd) => match CString::new(fd) {
                    Ok(cstring) => {
                        *output = cstring.into_raw();
                        cbinding::error_code::ok
                    }
                    Err(_) => cbinding::error_code::null_error,
                },
                _ => cbinding::error_code::unavailable_error,
            },
        },
        Err(err) => err_code(err),
    }
}

/// \brief Output of `get_device_all_core_status`
#[repr(C)]
pub struct CoreStatusPair {
    pub core_index: u8,
    pub status: CoreStatus,
}

/// \brief Examine each core of the device, whether it is available or not.
///
/// \remark output buffer must be allocated from outside of FFI boundary,
/// and retrieved array of `CoreStatusPair` must be destroyed by `destroy_core_status_pair`
///
/// @param handle device_handle of Furiosa NPU device.
/// @param[out] output output buffer for the array of `CoreStatusPair`.
/// @param[out] output_len output buffer for length of array.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_all_core_status(
    handle: device_handle,
    output: *mut *mut CoreStatusPair,
    output_len: *mut u8,
) -> cbinding::error_code {
    ffi_helpers::null_pointer_check!(handle, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, cbinding::error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output_len, cbinding::error_code::invalid_input);
    let device = device_mut(handle);
    match blocking::get_status_all(device) {
        Ok(hashmap) => {
            let mut output_vec: Vec<CoreStatusPair> = hashmap
                .iter()
                .map(|(idx, status)| CoreStatusPair {
                    core_index: *idx,
                    status: transform_core_status(status.clone()),
                })
                .collect();
            output_vec.shrink_to_fit();
            *output = output_vec.as_mut_ptr();
            *output_len = output_vec.len() as u8;
            mem::forget(output_vec);
            cbinding::error_code::ok
        }
        Err(err) => err_code(err),
    }
}

/// \brief Safely free array of `CoreStatusPair` that is allocated by `get_device_all_core_status`.
///
/// @param raw pointer to array of `CoreStatusPair`.
/// @param len length of array.
#[no_mangle]
pub unsafe extern "C" fn destroy_core_status_pair(raw: *mut CoreStatusPair, len: u8) {
    ffi_helpers::null_pointer_check!(raw);
    let vec = Vec::from_raw_parts(raw, len as usize, len as usize);
    drop(vec)
}

/// \brief Represent a core range type
#[repr(C)]
pub enum CoreRangeType {
    All,
    Range,
}

/// \brief Represent a core range
#[repr(C)]
pub struct CoreRange {
    pub range_type: CoreRangeType,
    pub start: u8,
    pub end: u8,
}

/// \brief Represent a core status
#[repr(C)]
pub enum CoreStatus {
    Available,
    Occupied,
    Unavailable,
}

/// \brief Safely free rust string that is represented in C string.
///
/// @param ptr pointer to rust string.
#[no_mangle]
pub unsafe extern "C" fn free_string(ptr: *const c_char) {
    let c_str = CString::from_raw(ptr as *mut _);
    drop(c_str)
}
