mod builder;
mod env;
mod inner;

use furiosa_device::DeviceConfig;
use pyo3::prelude::*;

use builder::DeviceConfigBuilderACPy;
use env::EnvBuilderNotDeterminedPy;

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
    #[staticmethod]
    pub fn warboy() -> DeviceConfigBuilderACPy {
        DeviceConfigBuilderACPy::new(DeviceConfig::warboy())
    }

    #[staticmethod]
    pub fn from_env(key: &str) -> EnvBuilderNotDeterminedPy {
        EnvBuilderNotDeterminedPy::new(DeviceConfig::from_env(key))
    }
}
