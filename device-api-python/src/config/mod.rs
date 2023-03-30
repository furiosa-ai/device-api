use furiosa_device::{DeviceConfig};
use pyo3::prelude::*;

use crate::builder::*;

#[pyclass(name = "DeviceConfig")]
pub struct DeviceConfigPy {
    inner : DeviceConfig
}

impl DeviceConfigPy {
    pub fn new(d: DeviceConfig) -> Self {
        Self { inner: d }
    }
}

#[pymethods]
impl DeviceConfigPy {
    pub fn warboy() -> DeviceConfigBuilder<String, NotDetermined, NotDetermined> {

    }
}