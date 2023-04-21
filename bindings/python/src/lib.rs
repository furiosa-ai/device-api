use furiosa_device::{find_devices, get_device, list_devices};
use pyo3::prelude::*;

mod arch;
mod config;
mod device;
mod errors;
mod hwmon;
mod sync;
use arch::ArchPy;
use config::DeviceConfigPy;
use device::{DeviceFilePy, DeviceModePy, DevicePy};
use errors::to_py_err;

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

#[pyfunction(name = "get_device")]
fn get_device_python(py: Python<'_>, device_name: String) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        get_device(device_name)
            .await
            .map(DeviceFilePy::new)
            .map_err(to_py_err)
    })
}

#[pymodule]
#[pyo3(name = "furiosa_device")]
fn furiosa_device_python(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<ArchPy>()?;
    m.add_class::<DevicePy>()?;
    m.add_class::<DeviceModePy>()?;
    m.add_class::<DeviceConfigPy>()?;
    m.add_function(wrap_pyfunction!(list_devices_python, m)?)?;
    m.add_function(wrap_pyfunction!(find_devices_python, m)?)?;
    m.add_function(wrap_pyfunction!(get_device_python, m)?)?;

    let sync_module = pyo3::wrap_pymodule!(sync::furiosa_device_python_sync);
    m.add_wrapped(sync_module)?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("furiosa_device.sync", m.getattr("furiosa_device_sync")?)?;
    Ok(())
}
