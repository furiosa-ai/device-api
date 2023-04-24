use std::collections::HashMap;

use furiosa_device::blocking::{find_devices, get_device, get_status_all, list_devices};
use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::device::{CoreStatusPy, DeviceFilePy, DevicePy};
use crate::errors::to_py_err;
use crate::hwmon::{FetcherPy, SensorValuePy};
use crate::DeviceConfigPy;

#[pyclass(extends=DevicePy)]
struct DeviceSyncPy {
    runtime: Runtime,
}

impl DeviceSyncPy {
    fn new() -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
        }
    }
}

#[pymethods]
impl DeviceSyncPy {
    fn get_status_core(self_: PyRef<'_, Self>, core: u8) -> PyResult<CoreStatusPy> {
        self_
            .runtime
            .block_on(self_.as_ref().inner.get_status_core(core))
            .map(CoreStatusPy::new)
            .map_err(to_py_err)
    }

    fn get_status_all(self_: PyRef<'_, Self>) -> PyResult<HashMap<u8, CoreStatusPy>> {
        get_status_all(&self_.as_ref().inner)
            .map(|m| {
                m.into_iter()
                    .map(|(k, v)| (k, CoreStatusPy::new(v)))
                    .collect()
            })
            .map_err(to_py_err)
    }

    fn get_hwmon_fetcher(self_: PyRef<'_, Self>, py: Python<'_>) -> Py<PyAny> {
        let fetcher = self_.as_ref().get_hwmon_fetcher();
        let initializer = PyClassInitializer::from(fetcher).add_subclass(FetcherSyncPy::new());
        Py::new(py, initializer).unwrap().into_py(py)
    }
}

#[pyclass(extends=FetcherPy)]
struct FetcherSyncPy {
    runtime: Runtime,
}

impl FetcherSyncPy {
    fn new() -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
        }
    }
}

#[pymethods]
impl FetcherSyncPy {
    fn read_currents(self_: PyRef<'_, Self>) -> PyResult<Vec<SensorValuePy>> {
        self_
            .runtime
            .block_on(self_.as_ref().inner.read_currents())
            .map(|vec| {
                vec.iter()
                    .map(SensorValuePy::new)
                    .collect::<Vec<SensorValuePy>>()
            })
            .map_err(to_py_err)
    }

    fn read_voltages(self_: PyRef<'_, Self>) -> PyResult<Vec<SensorValuePy>> {
        self_
            .runtime
            .block_on(self_.as_ref().inner.read_voltages())
            .map(|vec| {
                vec.iter()
                    .map(SensorValuePy::new)
                    .collect::<Vec<SensorValuePy>>()
            })
            .map_err(to_py_err)
    }

    fn read_powers_average(self_: PyRef<'_, Self>) -> PyResult<Vec<SensorValuePy>> {
        self_
            .runtime
            .block_on(self_.as_ref().inner.read_powers_average())
            .map(|vec| {
                vec.iter()
                    .map(SensorValuePy::new)
                    .collect::<Vec<SensorValuePy>>()
            })
            .map_err(to_py_err)
    }

    fn read_temperatures(self_: PyRef<'_, Self>) -> PyResult<Vec<SensorValuePy>> {
        self_
            .runtime
            .block_on(self_.as_ref().inner.read_temperatures())
            .map(|vec| {
                vec.iter()
                    .map(SensorValuePy::new)
                    .collect::<Vec<SensorValuePy>>()
            })
            .map_err(to_py_err)
    }
}

#[pyfunction(name = "list_devices_sync")]
fn list_devices_python_sync(py: Python<'_>) -> PyResult<Vec<Py<PyAny>>> {
    let mut device_syncs = vec![];
    for device in list_devices().unwrap() {
        let initializer =
            PyClassInitializer::from(DevicePy::new(device)).add_subclass(DeviceSyncPy::new());
        let device_sync_py = Py::new(py, initializer).unwrap().into_py(py);
        device_syncs.push(device_sync_py);
    }
    Ok(device_syncs)
}

#[pyfunction(name = "find_devices_sync")]
fn find_devices_python_sync(config: DeviceConfigPy) -> PyResult<Vec<DeviceFilePy>> {
    find_devices(&config.inner)
        .map(|vec| {
            vec.into_iter()
                .map(DeviceFilePy::new)
                .collect::<Vec<DeviceFilePy>>()
        })
        .map_err(to_py_err)
}

#[pyfunction(name = "get_device_sync")]
fn get_device_python_sync(device_name: String) -> PyResult<DeviceFilePy> {
    get_device(device_name)
        .map(DeviceFilePy::new)
        .map_err(to_py_err)
}

#[pymodule]
#[pyo3(name = "furiosa_device_sync")]
pub fn furiosa_device_python_sync(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(list_devices_python_sync, m)?)?;
    m.add_function(wrap_pyfunction!(find_devices_python_sync, m)?)?;
    m.add_function(wrap_pyfunction!(get_device_python_sync, m)?)?;

    Ok(())
}
