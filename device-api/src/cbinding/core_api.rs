use libc::c_char;

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum furiReturn_t {
    ok = 0,
    // define errors
    unknown_error,
}

// Initialization API
pub fn furiInit() -> furiReturn_t {
    furiReturn_t::ok
}
pub fn furiShutdown() -> furiReturn_t {
    furiReturn_t::ok
}

// System API
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum furiArch_t {
    warboy = 0,
    rngd,
    rngd_max,
    rngd_s,
}
const FURI_MAX_HW_METADATA_SIZE: usize = 25;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct furiDriverVersion_t {
    pub arch: furiArch_t,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub metadata: [c_char; FURI_MAX_HW_METADATA_SIZE], //hash
}

const FURI_MAX_DRIVER_INFO_SIZE: usize = 24; //임의의 값
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct furiSystemDriverInfo_t {
    pub count: u8,
    pub driverInfo: [furiDriverVersion_t; FURI_MAX_DRIVER_INFO_SIZE],
}

pub fn furiSystemGetDriverInfo(systemDriverInfo: *mut furiSystemDriverInfo_t) -> furiReturn_t {
    furiReturn_t::ok
}

pub fn furiSystemGetSrIovCapability(supported: *mut bool) -> furiReturn_t {
    furiReturn_t::ok
}

const FURI_MAX_DEVICE_BDF_SIZE: usize = 64;

pub struct furiSystemPhysicalDevice_t {
    bdf: [c_char; FURI_MAX_DEVICE_BDF_SIZE],
}

#[allow(non_camel_case_types)]
pub type furiSystemPhysicalDeviceHandle_t = furiSystemPhysicalDevice_t;

const FURI_MAX_DRIVER_INFO_DEVICE_SIZE: usize = 64;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct furiSystemPhysicalDeviceHandles_t {
    pub count: u8,
    pub device_handles: [furiSystemPhysicalDeviceHandle_t; FURI_MAX_DRIVER_INFO_DEVICE_SIZE],
}

pub fn furiSystemGetPhysicalDeviceHandles(
    devices: *mut furiSystemPhysicalDeviceHandles_t,
) -> furiReturn_t {
    furiReturn_t::ok
}

pub fn furiSystemGetPhysicalDeviceSrIovCapability(
    handle: furiSystemPhysicalDeviceHandle_t,
    supported: *mut bool,
) -> furiReturn_t {
    furiReturn_t::ok
}

pub fn furiSystemGetPhysicalDeviceMaxVfNum(
    handle: furiSystemPhysicalDeviceHandle_t,
    num: *mut u8,
) -> furiReturn_t {
    furiReturn_t::ok
}

pub fn furiSystemGetPhysicalDeviceVfConfig(
    handle: furiSystemPhysicalDeviceHandle_t,
    vfNum: *mut u8,
) -> furiReturn_t {
    furiReturn_t::ok
}

pub fn furiSystemConfigurePhysicalDeviceVf(
    handle: furiSystemPhysicalDeviceHandle_t,
    num: u8,
) -> furiReturn_t {
    furiReturn_t::ok
}

pub fn furiSystemUnconfigurePhysicalDeviceVf(
    handle: furiSystemPhysicalDeviceHandle_t,
) -> furiReturn_t {
    furiReturn_t::ok
}

// Device Info API
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct furiDeviceInfoDevice_t {
    bdf: [c_char; FURI_MAX_DEVICE_BDF_SIZE],
}

pub type furiDeviceInfoDeviceHandle_t = furiDeviceInfoDevice_t;

const FURI_MAX_DEVICE_HANDLE_SIZE: usize = 64;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct furiDeviceInfoDeviceHandles_t {
    pub count: u8,
    pub device_handles: [furiDeviceInfoDevice_t; FURI_MAX_DEVICE_HANDLE_SIZE],
}

pub fn furiDeviceInfoGetDeviceHandle(info: *mut furiDeviceInfoDeviceHandles_t) -> furiReturn_t {
    furiReturn_t::ok
}

pub fn furiDeviceInfoGetDeviceHandleByUUID(
    uuid: *const c_char,
    handle: *mut furiDeviceInfoDevice_t,
) -> furiReturn_t {
    furiReturn_t::ok
}

const FURI_MAX_BUFFER_SIZE: usize = 96;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct furiDeviceInfo_t {
    pub arch: furiArch_t,
    pub name: [c_char; FURI_MAX_BUFFER_SIZE],
    pub serial: [c_char; FURI_MAX_BUFFER_SIZE],
    pub uuid: [c_char; FURI_MAX_BUFFER_SIZE],
    pub core_num: u32,
}

pub fn furiDeviceInfoGetDeviceInfo(
    handle: furiDeviceInfoDevice_t,
    info: *mut furiDeviceInfo_t,
) -> furiReturn_t {
    furiReturn_t::ok
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct furiDeviceFirmwareVersion_t {
    pub arch: furiArch_t,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub metadata: [c_char; FURI_MAX_HW_METADATA_SIZE],
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct furiDeviceInfoDeviceHwInfo_t {
    pub bdf: [c_char; FURI_MAX_BUFFER_SIZE],
    pub pci_dev_id: [c_char; FURI_MAX_BUFFER_SIZE],
    pub firmware_version: furiDeviceFirmwareVersion_t,
    pub driver_version: furiDriverVersion_t,
    pub numa_node: u32,
}

pub fn furiDeviceInfoGetDeviceHwInfo(
    handle: furiDeviceInfoDevice_t,
    info: *mut furiDeviceInfoDeviceHwInfo_t,
) -> furiReturn_t {
    furiReturn_t::ok
}


// Observatory API(perf_counter, hwmon)

