use std::ffi::{CString};
use libc::c_char;
use tokio::runtime::Runtime;
use furiosa_device;

use crate::{arch, device_handle, err_code, error_code};
use crate::arch::Arch;



unsafe fn device_mut(handle: device_handle) -> &'static mut furiosa_device::Device {
    handle.as_mut().expect("invalid device_handle pointer")

}

#[no_mangle]
pub unsafe extern "C" fn get_device_name(handle: device_handle, output: *mut *mut c_char) -> error_code{
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    let device = device_mut(handle);
    match CString::new(device.name()) {
        Ok(cstring) => {
            *output = cstring.into_raw();
            error_code::ok
        },
        Err(_) => error_code::null_error,
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_device_index(handle: device_handle, output: *mut u8) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    let device = device_mut(handle);
    *output = device.device_index();
    error_code::ok
}

#[no_mangle]
pub unsafe extern "C" fn get_device_arch(handle: device_handle, output: *mut Arch) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    let device = device_mut(handle);
    match device.arch() {
        furiosa_device::Arch::WarboyA0 => {*output = Arch::WarboyA0},
        furiosa_device::Arch::WarboyB0 => {*output = Arch::WarboyB0},
        furiosa_device::Arch::Renegade => {*output = Arch::Renegade},
        furiosa_device::Arch::U250 => {*output = Arch::U250},
    }
    error_code::ok
}


#[no_mangle]
pub unsafe extern "C" fn get_device_aliveness(handle: device_handle, output: *mut bool) -> error_code{
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    let device = device_mut(handle);
    match device.alive() {
        Ok(alive) => {
            *output = alive;
            error_code::ok
        },
        Err(err) => err_code(err),
    }
}

#[repr(C)]
pub struct ErrorStatesKeyValuePair {
    pub key: *const c_char,
    pub value: u32,
}

#[no_mangle]
pub unsafe extern "C" fn get_device_error_states(handle: device_handle, output: *mut *mut ErrorStatesKeyValuePair, output_len: *mut usize) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output_len, error_code::invalid_input);
    let device = device_mut(handle);
    match device.atr_error() {
        Ok(hashMap) => {
            let mut output_vec = Vec::new();
            for(k, v) in hashMap.iter() {
                output_vec.push(ErrorStatesKeyValuePair{
                    key: CString::new(k.clone()).unwrap().into_raw(),
                    value: *v,
                })
            }

            output_vec.shrink_to_fit();
            *output = output_vec.as_mut_ptr();
            *output_len = output_vec.len();
            error_code::ok
        },
        Err(err) => err_code(err)
    }
}

#[no_mangle]
pub unsafe extern "C" fn destroy_error_states(raw:*mut ErrorStatesKeyValuePair, len: u8) {
    ffi_helpers::null_pointer_check!(raw);
    let vec = Vec::from_raw_parts(raw, len as usize, len as usize);
    drop(vec);
}

#[no_mangle]
pub unsafe extern "C" fn get_device_pci_bus_number(handle: device_handle, output: *mut *mut c_char) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    let device = device_mut(handle);
    match device.busname() {
        Ok(busname) => match CString::new(busname) {
            Ok(csting) => {
                *output = csting.into_raw();
                error_code::ok
            },
            Err(_) => error_code::null_error
        }
        Err(err) => err_code(err)
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_device_pci_dev_id(handle: device_handle, output: *mut *mut c_char) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    let device = device_mut(handle);
    match device.pci_dev() {
        Ok(pci_dev_id) => match CString::new(pci_dev_id) {
            Ok(cstring) => {
                *output = cstring.into_raw();
                error_code::ok
            },
            Err(_) => error_code::null_error,
        }
        Err(err) => err_code(err),
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_device_serial_number(handle: device_handle, output: *mut *mut c_char) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    let device = device_mut(handle);
    match device.device_sn() {
        Ok(serial_number) => match CString::new(serial_number) {
            Ok(cstring) => {
                *output = cstring.into_raw();
                error_code::ok
            },
            Err(_) => error_code::null_error,
        }
        Err(err) => err_code(err)
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_device_uuid(handle: device_handle, output: *mut *mut c_char) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    let device = device_mut(handle);
    match device.device_uuid() {
        Ok(uuid) => match CString::new(uuid) {
            Ok(cstring) => {
                *output = cstring.into_raw();
                error_code::ok
            },
            Err(_) => error_code::null_error,
        }
        Err(err) => err_code(err)
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_device_firmware_version(handle: device_handle, output: *mut *mut c_char) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    let device = device_mut(handle);
    match device.firmware_version() {
        Ok(firmware_version) => match CString::new(firmware_version) {
            Ok(cstring) => {
                *output = cstring.into_raw();
                error_code::ok
            },
            Err(_) => error_code::null_error,
        }
        Err(err) => err_code(err)
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_device_driver_version(handle: device_handle, output: *mut *mut c_char) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    let device = device_mut(handle);
    match device.driver_version() {
        Ok(driver_version) => match CString::new(driver_version) {
            Ok(cstring) => {
                *output = cstring.into_raw();
                error_code::ok
            },
            Err(_) => error_code::null_error,
        }
        Err(err) => err_code(err)
    }
}


#[no_mangle]
pub unsafe extern "C" fn get_device_heartbeat(handle: device_handle, output: *mut u32) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    let device = device_mut(handle);
    match device.heartbeat() {
        Ok(heartbeat) => {
            *output = heartbeat;
            error_code::ok
        },
        Err(err) => err_code(err)
    }
}




#[repr(C)]
pub struct ClockFrequency {
    pub name: *const c_char,
    pub unit: *const c_char,
    pub value: u32
}

#[no_mangle]
pub unsafe extern "C" fn get_device_clock_frequency(handle: device_handle, output: *mut *mut ClockFrequency, output_len: *mut u8) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output_len, error_code::invalid_input);
    let device = device_mut(handle);
    match device.clock_frequency() {
        Ok(clockFrequency) => {
            let mut output_vec = Vec::new();
            for c in clockFrequency.iter() {
                output_vec.push(ClockFrequency{
                    name: CString::new(c.name()).unwrap().into_raw(),
                    unit: CString::new(c.unit()).unwrap().into_raw(),
                    value: c.value(),
                })
            }

            output_vec.shrink_to_fit();
            *output = output_vec.as_mut_ptr();
            *output_len = output_vec.len() as u8;
            error_code::ok
        },
        Err(err) => err_code(err)
    }
}

#[no_mangle]
pub unsafe extern "C" fn destroy_clock_frequency(raw:*mut ClockFrequency, len: u8)  {
    ffi_helpers::null_pointer_check!(raw);
    let vec = Vec::from_raw_parts(raw, len as usize, len as usize);
    drop(vec);
}

#[no_mangle]
pub unsafe extern "C" fn get_device_numanode(handle: device_handle, output: *mut u8) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    let device = device_mut(handle);
    match device.numa_node() {
        Ok(numanode) => match numanode {
            furiosa_device::NumaNode::UnSupported => error_code::unsupported_error,
            furiosa_device::NumaNode::Id(idx) => {
                *output = idx as u8;
                error_code::ok
            },
        }
        Err(err) => err_code(err)
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_device_core_num(handle: device_handle, output: *mut u8) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    let device = device_mut(handle);
    *output = device.core_num();
    error_code::ok
}

#[no_mangle]
pub unsafe extern "C" fn get_device_cores(handle: device_handle, output: *mut *mut u8, output_len: *mut u8) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output_len, error_code::invalid_input);
    let device = device_mut(handle);
    let mut cores = device.cores().clone();
    cores.shrink_to_fit();
    *output = cores.as_mut_ptr();
    *output_len = cores.len() as u8;
    error_code::ok
}

#[repr(C)]
pub struct DeviceFile {
    pub device_index: u8,
    pub core_range: CoreRange,
    pub path: *const c_char,
    pub mode: DeviceMode,
}

fn transform_device_file(origin: &furiosa_device::DeviceFile) -> DeviceFile {
    DeviceFile{
        device_index: 0,
        core_range: match origin.core_range() {
            furiosa_device::CoreRange::All => {
                CoreRange{
                    range_type: CoreRangeType::All,
                    // It would be nice, I we could fill real values here...
                    start: 0,
                    end: 0,
                }
            }
            furiosa_device::CoreRange::Range(r) => {
                CoreRange{
                    range_type: CoreRangeType::Range,
                    start: r.0,
                    end: r.1,
                }
            }
        },
        path: CString::new(origin.path().as_path().to_str().unwrap()).unwrap().into_raw(),
        mode: match origin.mode() {
            furiosa_device::DeviceMode::Single => DeviceMode::Single,
            furiosa_device::DeviceMode::Fusion => DeviceMode::Fusion,
            furiosa_device::DeviceMode::MultiCore=> DeviceMode::MultiCore,
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_device_files(handle: device_handle, output: *mut *mut DeviceFile, output_len: *mut u8) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output_len, error_code::invalid_input);
    let device = device_mut(handle);
    let mut deviceFilesVec: Vec<DeviceFile> = device.dev_files().iter().map(|f| transform_device_file(f)).collect();
    deviceFilesVec.shrink_to_fit();
    *output = deviceFilesVec.as_mut_ptr();
    *output_len = deviceFilesVec.len() as u8;
    error_code::ok
}

#[no_mangle]
pub unsafe extern "C" fn destroy_device_files(raw: *mut DeviceFile, len: u8) {
    ffi_helpers::null_pointer_check!(raw);
    let vec = Vec::from_raw_parts(raw, len as usize, len as usize);
    drop(vec);
}

#[allow(non_camel_case_types)]
pub type performance_counter_handle = *mut furiosa_device::perf_regs::PerformanceCounter;

#[repr(C)]
pub struct PerformanceCounterPair {
    pub DeviceFile: DeviceFile,
    pub PerformanceCounterHandle: performance_counter_handle,
}

#[no_mangle]
//bg: golang쪽에서 performance_counter를 쓸일이 있을지 모르겠음, performance handler바인딩은 일단 보류
pub unsafe extern "C" fn get_device_performance_counters(handle: device_handle, output: *mut *mut PerformanceCounterPair, output_len: *mut u8) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output_len, error_code::invalid_input);
    let device = device_mut(handle);
    let mut perf_counters: Vec<PerformanceCounterPair> = device.performance_counters()
        .iter()
        .map(
            |pair|{
                let deviceFile = pair.0;
                let performanceCounter = pair.1;
                PerformanceCounterPair{
                    DeviceFile: transform_device_file(deviceFile),
                    //bg: how to free the boxed pointer?
                    PerformanceCounterHandle: Box::into_raw(Box::new(performanceCounter)),
                }
            }
        ).collect();

    perf_counters.shrink_to_fit();
    *output = perf_counters.as_mut_ptr();
    *output_len = perf_counters.len() as u8;
    error_code::ok
}

#[no_mangle]
pub unsafe extern "C" fn destroy_performance_counters(raw: *mut PerformanceCounterPair, len: u8) {
    ffi_helpers::null_pointer_check!(raw);
    let vec = Vec::from_raw_parts(raw, len as usize, len as usize);

    //bg: need to check whether this is the right way to free boxed pointer
    for pair in vec.iter() {
        drop(Box::from_raw(pair.PerformanceCounterHandle))
    }
    drop(vec)
}


fn transform_core_status(origin: furiosa_device::CoreStatus) -> CoreStatus {
    match origin {
        furiosa_device::CoreStatus::Available => CoreStatus::Available,
        furiosa_device::CoreStatus::Occupied(_) => CoreStatus::Occupied,
        furiosa_device::CoreStatus::Unavailable => CoreStatus::Unavailable,
    }
}

//bg: 원본 함수의 경우 결과 반환용 열거형에서 Occupied는 데이터를 가진 필드이나 C에선 두개의 함수로 분리(get_device_core_occupied_fd)
#[no_mangle]
pub unsafe extern "C" fn get_device_core_status(handle: device_handle, core_idx: u8, output: *mut CoreStatus) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    let device = device_mut(handle);
    let status = Runtime::new().unwrap().block_on(device.get_status_core(core_idx));
    match status {
        Ok(coreStatus) => {
            *output = transform_core_status(coreStatus);
            error_code::ok
        }
        Err(err) => err_code(err)
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_device_core_occupied_fd(handle: device_handle, core_idx: u8, output: *mut *mut c_char) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    let device = device_mut(handle);
    let status = Runtime::new().unwrap().block_on(device.get_status_core(core_idx));
    match status {
        Ok(coreStatus) =>
            match coreStatus {
                furiosa_device::CoreStatus::Occupied(fd) => match CString::new(fd){
                    Ok(cstring) => {
                        *output = cstring.into_raw();
                        error_code::ok
                    },
                    Err(_) => error_code::null_error
                },
                _ => error_code::unavailable_error,
            }
        Err(err) => err_code(err)
    }
}


#[repr(C)]
pub struct CoreStatusPair {
    pub core_index: u8,
    pub status: CoreStatus,
}

//bg: 개인적으로 이api가 C나 golang에서 필요한지는 의문, 필요없을시 삭제해도 될거같음
#[no_mangle]
pub unsafe extern "C" fn get_device_all_core_status(handle: device_handle, output: *mut *mut CoreStatusPair, output_len: *mut u8) -> error_code {
    ffi_helpers::null_pointer_check!(handle, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output, error_code::invalid_input);
    ffi_helpers::null_pointer_check!(output_len, error_code::invalid_input);
    let device = device_mut(handle);
    let result = Runtime::new().unwrap().block_on(device.get_status_all());

    match result {
        Ok(hashMap) => {
            let mut output_vec :Vec<CoreStatusPair> = hashMap.iter().map(|(idx, status)| {
                CoreStatusPair{ core_index: idx.clone(), status: transform_core_status( status.clone())}
            }).collect();
            output_vec.shrink_to_fit();
            *output = output_vec.as_mut_ptr();
            *output_len = output_vec.len() as u8;
            error_code::ok
        }
        Err(err) => err_code(err)
    }
}

#[no_mangle]
pub unsafe extern "C" fn destroy_core_status_pair(raw: *mut CoreStatusPair, len: u8) {
    ffi_helpers::null_pointer_check!(raw);
    let vec = Vec::from_raw_parts(raw, len as usize, len as usize);
    drop(vec)
}


//bg: 일단 이거는 추후에 필요하다 싶으면 진행
//get_hwmon_fetcher

#[repr(C)]
pub enum CoreRangeType {
    All,
    Range,
}

#[repr(C)]
pub struct  CoreRange {
    pub range_type: CoreRangeType,
    pub start: u8,
    pub end: u8,

}

#[repr(C)]
pub enum DeviceMode {
    Single,
    Fusion,
    MultiCore,
}

#[repr(C)]
pub struct DeviceConfig {
    arch: arch::Arch,
    mode: DeviceMode,
    count: u8,
}

#[repr(C)]
pub enum CoreStatus {
    Available,
    Occupied,
    Unavailable,
}