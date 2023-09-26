use ffi_helpers;
use tokio::runtime::Runtime;
use furiosa_device::{DeviceError, Device};


mod arch;
mod device;
mod hwmon;

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum error_code {
    ok = 0,
    //...errors for edge case
    invalid_input,
    null_error,
    unsupported_error, //only for numa
    unavailable_error, // the requested operation is not available
    //...from DeviceResult
    device_not_found,
    device_busy,
    io_error,
    permission_denied_error,
    unknown_arch_error,
    incompatible_driver_error,
    hwmon_error,
    performance_counter_error,
    unexpected_value_error,
    parse_error,
    //...unknown
    unknown_error

}

fn err_code (err: DeviceError) -> error_code {
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

#[allow(non_camel_case_types)]
pub type device_handle = *mut Device;

// pub async fn list_devices() -> DeviceResult<Vec<Device>>
//bg: list_devices로 얻어진 device_handle은 무조건 destory_device_handles로 정리되어야함
#[no_mangle]
pub unsafe extern "C" fn list_devices(output: *mut *mut device_handle, output_len: *mut u8) -> error_code {
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output_len, error_code::invalid_input);
    let result = Runtime::new().unwrap().block_on(furiosa_device::list_devices());
    match result {
        Ok(vec) => {
            let mut output_vec: Vec<device_handle> = vec.iter().map(|d|Box::into_raw(Box::new(d.clone()))).collect();
            output_vec.shrink_to_fit();
            *output = output_vec.as_mut_ptr();
            *output_len = output_vec.len() as u8;
            error_code::ok
        }
        Err(err) => err_code(err)
    }
}

#[no_mangle]
pub unsafe extern "C" fn destroy_device_handles(raw: *mut device_handle, len: u8) {
    ffi_helpers::null_pointer_check!(raw);
    let vec = Vec::from_raw_parts(raw, len as usize, len as usize);
    drop(vec)

}

// pub async fn get_device(idx: u8) -> DeviceResult<Device>
#[no_mangle]
pub unsafe extern "C" fn get_device(idx: u8, output: *mut device_handle) -> error_code {
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    let result = Runtime::new().unwrap().block_on(furiosa_device::get_device(idx));
    match result {
        Ok(device) => {
            *output = Box::into_raw(Box::new(device));
            error_code::ok
        }
        Err(err) => err_code(err)
    }
}

#[no_mangle]
pub unsafe extern "C" fn destroy_device_handle(device: device_handle) {
    ffi_helpers::null_pointer_check!(device);
    let boxed = Box::from_raw(device);
    drop(boxed)
}


//TBD
/*
// pub async fn find_device_files(config: &DeviceConfig) -> DeviceResult<Vec<DeviceFile>>
#[no_mangle]
pub extern "C" fn find_device_files(device_config: *const device::DeviceConfig) -> *mut device::DeviceFileList {
    ffi_helpers::null_pointer_check!(device_config);

    ptr::null_mut()
}

// pub async fn get_device_file<S: AsRef<str>>(device_name: S) -> DeviceResult<DeviceFile>
#[no_mangle]
pub extern "C" fn get_device_file(device_name: *const char) -> *mut device::DeviceFile {
    ffi_helpers::null_pointer_check!(device_name);

    ptr::null_mut()
}
*/