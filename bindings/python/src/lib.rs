use furiosa_device::{find_devices, get_device, get_device_file, list_devices};
use hwmon::FetcherPy;
use pyo3::prelude::*;

mod arch;
mod config;
mod device;
mod errors;
mod hwmon;
mod sync;
use arch::ArchPy;
use config::DeviceConfigPy;
use device::{
    ClockFrequencyPy, CoreRangePy, DeviceFilePy, DeviceModePy, DevicePy, PerformanceCounterPy,
    UtilizationPy,
};
use errors::to_py_err;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_SHORT_HASH: &str = env!("FURIOSA_GIT_SHORT_HASH");
pub const BUILD_TIMESTAMP: &str = env!("FURIOSA_BUILD_TIMESTAMP");

/// `list_devices` enumerates all Furiosa NPU devices in the system.
/// One can simply call as below:
/// ```python
/// import asyncio
/// from furiosa_device import list_devices
///
/// async def main():
///     devices = await furiosa_device.list_devices()
/// asyncio.run(main())
/// ```
///
/// `Device` offers methods for further information of each device.
#[pyfunction(name = "list_devices")]
fn list_devices_python(py: Python<'_>) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        list_devices()
            .await
            .map(|vec| {
                vec.into_iter()
                    .map(DevicePy::new)
                    .collect::<Vec<DevicePy>>()
            })
            .map_err(to_py_err)
    })
}

/// `get_device` returns a specific Furiosa NPU device in the system.
/// One can simply call as below:
/// ```python
/// import asyncio
/// from furiosa_device import get_device
///
/// async def main():
///     device = await furiosa_device.get_device(0)
/// asyncio.run(main())
/// ```
///
/// `Device` offers methods for further information of each device.
#[pyfunction(name = "get_device")]
fn get_device_python(py: Python<'_>, idx: u8) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        get_device(idx).await.map(DevicePy::new).map_err(to_py_err)
    })
}

/// If you have a desired configuration, call `find_devices` with your device configuration
/// described by a `DeviceConfig`. `find_devices` will return a list of
/// `DeviceFile`s if there are matched devices.
/// ```python
/// import asyncio
/// from furiosa_device import Arch, DeviceConfig, DeviceMode, find_devices
///
/// async def main():
///     // Find two Warboy devices, fused.
///     let config = furiosa_device.DeviceConfig(arch=Arch.Warboy, mode=DeviceMode.Fusion, count=2)
///     devices = await furiosa_device.find_devices(config)
/// asyncio.run(main())
/// ```
#[pyfunction(name = "find_devices")]
fn find_devices_python(py: Python<'_>, config: DeviceConfigPy) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        find_devices(&config.inner)
            .await
            .map(|vec| {
                vec.into_iter()
                    .map(DeviceFilePy::new)
                    .collect::<Vec<DeviceFilePy>>()
            })
            .map_err(to_py_err)
    })
}

/// In case you have prior knowledge on the system and want to pick out a
/// device file with specific name, use `get_device_file`.
/// ```python
/// import asyncio
/// from furiosa_device import get_device_file
///
/// async def main():
///     device_file = await get_device_file("npu0pe0")
/// ```
#[pyfunction(name = "get_device_file")]
fn get_device_file_python(py: Python<'_>, device_name: String) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        get_device_file(device_name)
            .await
            .map(DeviceFilePy::new)
            .map_err(to_py_err)
    })
}

#[pymodule]
#[pyo3(name = "furiosa_device")]
fn furiosa_device_python(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(list_devices_python, m)?)?;
    m.add_function(wrap_pyfunction!(get_device_python, m)?)?;
    m.add_function(wrap_pyfunction!(find_devices_python, m)?)?;
    m.add_function(wrap_pyfunction!(get_device_file_python, m)?)?;
    m.add_class::<DevicePy>()?;
    m.add_class::<DeviceFilePy>()?;
    m.add_class::<DeviceConfigPy>()?;
    m.add_class::<FetcherPy>()?;
    m.add_class::<CoreRangePy>()?;
    m.add_class::<ArchPy>()?;
    m.add_class::<DeviceModePy>()?;
    m.add_class::<ClockFrequencyPy>()?;
    m.add_class::<PerformanceCounterPy>()?;
    m.add_class::<UtilizationPy>()?;
    m.add("__version__", VERSION)?;
    m.add("__git_short_hash__", GIT_SHORT_HASH)?;
    m.add("__build_timestamp__", BUILD_TIMESTAMP)?;

    let sync_module = pyo3::wrap_pymodule!(sync::furiosa_device_python_sync);
    m.add_wrapped(sync_module)?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("furiosa_device.sync", m.getattr("furiosa_device_sync")?)?;
    Ok(())
}
