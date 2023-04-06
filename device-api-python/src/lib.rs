use device::DeviceFilePy;
use furiosa_device::{find_devices, get_device, list_devices};
use pyo3::prelude::*;

mod arch;
mod config;
mod device;
mod errors;
mod hwmon;

#[pyfunction(name = "list_devices")]
fn list_devices_python(py: Python) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        list_devices()
            .await
            .map(|list| {
                list.iter()
                    .map(|d| device::DevicePy::new(d.clone()))
                    .collect::<Vec<device::DevicePy>>()
            })
            .map_err(errors::to_py_err)
    })
}

#[pyfunction(name = "find_devices")]
fn find_devices_python(py: Python, config: config::DeviceConfigPy) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        find_devices(&config.inner)
            .await
            .map(|list| {
                list.iter()
                    .map(|d| device::DeviceFilePy::new(d.clone()))
                    .collect::<Vec<DeviceFilePy>>()
            })
            .map_err(errors::to_py_err)
    })
}

/// A Python module implemented in Rust.
#[pymodule]
fn furiosa_device_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<arch::ArchPy>();
    m.add_class::<device::DeviceModePy>();
    m.add_class::<config::DeviceConfigPy>();
    m.add_function(wrap_pyfunction!(list_devices_python, m)?)?;
    m.add_function(wrap_pyfunction!(find_devices_python, m)?)?;
    Ok(())
}
