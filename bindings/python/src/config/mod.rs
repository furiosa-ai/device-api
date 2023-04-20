use std::str::FromStr;

use furiosa_device::DeviceConfig;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyType;

use crate::errors::to_py_err;
use crate::{arch::ArchPy, device::DeviceModePy};

#[pyclass(name = "DeviceConfig")]
#[derive(Clone)]
pub struct DeviceConfigPy {
    pub inner: DeviceConfig,
}

impl DeviceConfigPy {
    fn new(d: DeviceConfig) -> Self {
        Self { inner: d }
    }
}

#[pymethods]
impl DeviceConfigPy {
    #[new]
    #[pyo3(signature = (arch=ArchPy::Warboy, mode=DeviceModePy::Fusion, count=1))]
    fn py_new(arch: ArchPy, mode: DeviceModePy, count: u8) -> PyResult<DeviceConfigPy> {
        let config = match arch {
            ArchPy::WarboyA0 => DeviceConfig::warboy_a0(),
            ArchPy::Warboy => DeviceConfig::warboy(),
            _ => {
                return Err(PyRuntimeError::new_err(format!(
                    "Invalid architecture: Not supported architecture '{:?}'",
                    arch
                )))
            }
        };
        let config = match mode {
            DeviceModePy::Single => config.single(),
            DeviceModePy::MultiCore => config.multicore(),
            DeviceModePy::Fusion => config.fused(),
        }
        .count(count);
        Ok(DeviceConfigPy::new(config))
    }

    #[classmethod]
    fn from_env(_cls: &PyType, key: &str) -> PyResult<DeviceConfigPy> {
        DeviceConfig::from_env(key)
            .build()
            .map(DeviceConfigPy::new)
            .map_err(to_py_err)
    }

    #[classmethod]
    fn from_str(_cls: &PyType, key: &str) -> PyResult<DeviceConfigPy> {
        DeviceConfig::from_str(key)
            .map(DeviceConfigPy::new)
            .map_err(to_py_err)
    }
}
