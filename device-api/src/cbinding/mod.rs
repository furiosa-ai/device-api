#![allow(clippy::missing_safety_doc)]
use std::ffi::CStr;
use std::mem;
use std::panic::AssertUnwindSafe;

use ffi_helpers;
use libc::c_char;

use crate::blocking::{get_device, get_device_file, list_devices};
use crate::{cbinding, Device, DeviceError};

mod arch;
pub(crate) mod device;
mod test;

#[macro_export]
macro_rules! catch_unwind {
    ($closure:expr) => {
        match std::panic::catch_unwind(AssertUnwindSafe($closure)) {
            Ok(res) => res,
            Err(_) => cbinding::error_code::unknown_error,
        }
    };
}
pub(crate) use catch_unwind;

#[allow(non_camel_case_types)]
pub type device_handle = *mut Device;

/// \brief Represent a return status
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum error_code {
    /// When a function call is successful.
    ok = 0,
    /// When a input is invalid.
    invalid_input,
    /// When a function call fails while constructing c string from invalid bytes containing the null byte.
    null_error,
    /// When a certain operation is not supported on the system.
    unsupported_error,
    /// When a certain operation is not available in the current situation.
    unavailable_error,
    /// When a device is not found with the given option.
    device_not_found,
    /// When a device state is busy.
    device_busy,
    /// When a certain operation failed by an unexpected io error.
    io_error,
    /// When a certain operation failed by a permission deny.
    permission_denied_error,
    /// When an arch is unknown.
    unknown_arch_error,
    /// When a driver is incompatible.
    incompatible_driver_error,
    /// When hwmon error is occurred
    hwmon_error,
    /// When performance counter error is occurred
    performance_counter_error,
    /// When a retrieved value is invalid.
    unexpected_value_error,
    /// When a unicode parsing is failed
    parse_error,
    /// When a reason is unknown
    unknown_error,
}

pub(crate) fn err_code(err: DeviceError) -> error_code {
    match err {
        DeviceError::DeviceNotFound { .. } => error_code::device_not_found,
        DeviceError::DeviceBusy { .. } => error_code::device_busy,
        DeviceError::IoError { .. } => error_code::io_error,
        DeviceError::PermissionDenied { .. } => error_code::permission_denied_error,
        DeviceError::UnknownArch { .. } => error_code::unknown_arch_error,
        DeviceError::IncompatibleDriver { .. } => error_code::incompatible_driver_error,
        DeviceError::HwmonError { .. } => error_code::hwmon_error,
        DeviceError::PerformanceCounterError { .. } => error_code::performance_counter_error,
        DeviceError::UnexpectedValue { .. } => error_code::unexpected_value_error,
        DeviceError::ParseError { .. } => error_code::parse_error,
    }
}

/// \brief Retrieve device_handle of all Furiosa NPU devices in the system.
///
/// \remark output buffer must be allocated from outside of FFI boundary,
/// and retrieved device_handles must be destroyed by `destroy_device_handles`.
///
/// @param[out] output output buffer for array of device_handle.
/// @param[out] output_len output buffer for length of array.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn list_all_devices(
    output: *mut *mut device_handle,
    output_len: *mut u8,
) -> error_code {
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output_len, error_code::invalid_input);

    catch_unwind!(|| {
        let result = list_devices();
        match result {
            Ok(vec) => {
                let mut output_vec: Vec<device_handle> = vec
                    .iter()
                    .map(|d| Box::into_raw(Box::new(d.clone())))
                    .collect();
                output_vec.shrink_to_fit();
                *output = output_vec.as_mut_ptr();
                *output_len = output_vec.len() as u8;
                mem::forget(output_vec);
                error_code::ok
            }
            Err(err) => err_code(err),
        }
    })
}

/// \brief Destroy array of device_handle returned by `list_all_devices`.
///
/// @param raw pointer to array of device_handles.
/// @param len length of array.
#[no_mangle]
pub unsafe extern "C" fn destroy_device_handles(raw: *mut device_handle, len: u8) {
    ffi_helpers::null_pointer_check!(raw);
    let vec = Vec::from_raw_parts(raw, len as usize, len as usize);
    drop(vec)
}

/// \brief Retrieve device_handle with a specific index of Furiosa NPU device in the system.
///
/// \remark output buffer must be allocated from outside of FFI boundary,
/// and retrieved device_handle must be destroyed by `destroy_device_handle`.
///
/// @param idx index of Furiosa NPU device.
/// @param[out] output output buffer for device_handle.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_by_index(idx: u8, output: *mut device_handle) -> error_code {
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    catch_unwind!(|| {
        let result = get_device(idx);
        match result {
            Ok(device) => {
                *output = Box::into_raw(Box::new(device));
                error_code::ok
            }
            Err(err) => err_code(err),
        }
    })
}

/// \brief Destroy device_handle returned by `get_device_by_index`.
///
/// @param device device_handle to destroy.
#[no_mangle]
pub unsafe extern "C" fn destroy_device_handle(device: device_handle) {
    ffi_helpers::null_pointer_check!(device);
    let boxed = Box::from_raw(device);
    drop(boxed)
}

/// \brief Retrieve DeviceFile with a specific name of Furiosa NPU device in the system.
///
/// \remark output buffer must be allocated from outside of FFI boundary,
/// and retrieved DeviceFile must be destroyed by `destroy_device_file`.
///
/// @parm device_name pointer to C string for a device name (e.g., npu0, npu0pe0, npu0pe0-1),
/// the name should be terminated by null character.
/// @param[out] output output buffer for DeviceFile.
/// @return error_code::ok if successful, see `error_code` for error cases.
#[no_mangle]
pub unsafe extern "C" fn get_device_file_by_name(
    device_name: *const c_char,
    output: *mut *mut device::DeviceFile,
) -> error_code {
    ffi_helpers::null_pointer_check!(device_name, error_code::invalid_input);
    catch_unwind!(|| {
        let cstr = CStr::from_ptr(device_name);
        let str = String::from_utf8_lossy(cstr.to_bytes()).to_string();

        let result = get_device_file(str);
        match result {
            Ok(file) => {
                let output_file = device::transform_device_file(&file);
                *output = Box::into_raw(Box::new(output_file));
                error_code::ok
            }
            Err(err) => err_code(err),
        }
    })
}

/// \brief Destroy DeviceFile returned by `get_device_file_by_name`.
///
/// @param raw pointer to `DeviceFile` to destroy.
#[no_mangle]
pub unsafe extern "C" fn destroy_device_file(raw: *mut device::DeviceFile) {
    ffi_helpers::null_pointer_check!(raw);
    let boxed = Box::from_raw(raw);
    drop(boxed)
}
