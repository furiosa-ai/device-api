mod builder;
mod env;
mod inner;

use std::str::FromStr;

use env::EnvBuilderNotDeterminedPy;
use furiosa_device::DeviceConfig;
use pyo3::prelude::*;

use crate::errors::to_py_err;
use crate::{arch::ArchPy, device::DeviceModePy};

#[pyclass(name = "DeviceConfig")]
#[derive(Clone)]
pub struct DeviceConfigPy {
    inner: DeviceConfig,
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

    #[staticmethod]
    pub fn from_env(key: &str) -> PyResult<DeviceConfigPy> {
        DeviceConfig::from_env(key)
            .build()
            .map(DeviceConfigPy::new)
            .map_err(to_py_err)
    }

    #[staticmethod]
    pub fn from_str(key: &str) -> PyResult<DeviceConfigPy> {
        DeviceConfig::from_str(key)
            .map(DeviceConfigPy::new)
            .map_err(to_py_err)
    }
}
