use libc::c_char;

mod core_api;

#[no_mangle]
pub unsafe extern "C" fn furiInit() -> core_api::furiReturn_t {
    core_api::furiInit()
}
#[no_mangle]
pub unsafe extern "C" fn furiShutdown() -> core_api::furiReturn_t {
    core_api::furiShutdown()
}

#[no_mangle]
pub unsafe extern "C" fn furiSystemGetDriverInfo(
    systemDriverInfo: *mut core_api::furiSystemDriverInfo_t,
) -> core_api::furiReturn_t {
    core_api::furiSystemGetDriverInfo(systemDriverInfo)
}

#[no_mangle]
pub unsafe extern "C" fn furiSystemGetSrIovCapability(
    supported: *mut bool,
) -> core_api::furiReturn_t {
    core_api::furiSystemGetSrIovCapability(supported)
}

#[no_mangle]
pub unsafe extern "C" fn furiSystemGetPhysicalDeviceInfo(
    devices: *mut core_api::furiSystemPhysicalDeviceHandles_t,
) -> core_api::furiReturn_t {
    core_api::furiSystemGetPhysicalDeviceHandles(devices)
}

#[no_mangle]
pub unsafe extern "C" fn furiSystemGetPhysicalDeviceSrIovCapability(
    handle: core_api::furiSystemPhysicalDeviceHandle_t,
    supported: *mut bool,
) -> core_api::furiReturn_t {
    core_api::furiSystemGetPhysicalDeviceSrIovCapability(handle, supported)
}

#[no_mangle]
pub unsafe extern "C" fn furiSystemGetPhysicalDeviceMaxVfNum(
    handle: core_api::furiSystemPhysicalDeviceHandle_t,
    num: *mut u8,
) -> core_api::furiReturn_t {
    core_api::furiSystemGetPhysicalDeviceMaxVfNum(handle, num)
}

#[no_mangle]
pub unsafe extern "C" fn furiSystemGetPhysicalDeviceVfConfig(
    handle: core_api::furiSystemPhysicalDeviceHandle_t,
    vfNum: *mut u8,
) -> core_api::furiReturn_t {
    core_api::furiSystemGetPhysicalDeviceVfConfig(handle, vfNum)
}

#[no_mangle]
pub unsafe extern "C" fn furiSystemConfigurePhysicalDeviceVf(
    handle: core_api::furiSystemPhysicalDeviceHandle_t,
    num: u8,
) -> core_api::furiReturn_t {
    core_api::furiSystemConfigurePhysicalDeviceVf(handle, num)
}

#[no_mangle]
pub unsafe extern "C" fn furiSystemUnconfigurePhysicalDeviceVf(
    handle: core_api::furiSystemPhysicalDeviceHandle_t,
) -> core_api::furiReturn_t {
    core_api::furiSystemUnconfigurePhysicalDeviceVf(handle)
}

// Device Info API
#[no_mangle]
pub unsafe extern "C" fn furiDeviceInfoGetDeviceHandle(
    info: *mut core_api::furiDeviceInfoDeviceHandles_t,
) -> core_api::furiReturn_t {
    core_api::furiDeviceInfoGetDeviceHandle(info)
}

#[no_mangle]
pub unsafe extern "C" fn furiDeviceInfoGetDeviceHandleByUUID(
    uuid: *const c_char,
    handle: *mut core_api::furiDeviceInfoDevice_t,
) -> core_api::furiReturn_t {
    core_api::furiDeviceInfoGetDeviceHandleByUUID(uuid, handle)
}

#[no_mangle]
pub unsafe extern "C" fn furiDeviceInfoGetDeviceInfo(
    handle: core_api::furiDeviceInfoDevice_t,
    info: *mut core_api::furiDeviceInfo_t,
) -> core_api::furiReturn_t {
    core_api::furiDeviceInfoGetDeviceInfo(handle, info)
}

#[no_mangle]
pub unsafe extern "C" fn furiDeviceInfoGetDeviceHwInfo(
    handle: core_api::furiDeviceInfoDevice_t,
    info: *mut core_api::furiDeviceInfoDeviceHwInfo_t,
) -> core_api::furiReturn_t {
    core_api::furiDeviceInfoGetDeviceHwInfo(handle, info)
}
