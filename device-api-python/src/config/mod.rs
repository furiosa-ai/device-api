mod builder;
mod env;
mod inner;

use std::str::FromStr;

use furiosa_device::DeviceConfig;
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
    pub fn new(d: DeviceConfig) -> Self {
        Self { inner: d }
    }
}

#[pymethods]
impl DeviceConfigPy {
    #[new]
    #[pyo3(signature = (arch=ArchPy::Warboy, mode=DeviceModePy::Fusion, count=1))]
    fn py_new(arch: ArchPy, mode: DeviceModePy, count: u8) -> Self {
        // Currently only Arch::Warboy is supported
        let config = DeviceConfig::warboy();
        let config = match mode {
            DeviceModePy::Single => config.single(),
            DeviceModePy::MultiCore => config.multicore(),
            DeviceModePy::Fusion => config.fused(),
        }
        .count(count);
        DeviceConfigPy::new(config)
    }

    #[classmethod]
    fn from_env(cls: &PyType, key: &str) -> PyResult<DeviceConfigPy> {
        DeviceConfig::from_env(key)
            .build()
            .map(DeviceConfigPy::new)
            .map_err(to_py_err)
    }

    #[classmethod]
    fn from_str(cls: &PyType, key: &str) -> PyResult<DeviceConfigPy> {
        DeviceConfig::from_str(key)
            .map(DeviceConfigPy::new)
            .map_err(to_py_err)
    }
}
