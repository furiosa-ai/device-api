use furiosa_device::{Arch, DeviceConfigBuilder, DeviceMode, NotDetermined};
use pyo3::prelude::*;

use super::DeviceConfigPy;

#[pyclass(name = "DeviceConfigBuilderAC")]
pub struct DeviceConfigBuilderACPy {
    inner: DeviceConfigBuilder<Arch, NotDetermined, u8>,
}

impl DeviceConfigBuilderACPy {
    pub fn new(d: DeviceConfigBuilder<Arch, NotDetermined, u8>) -> Self {
        Self { inner: d }
    }
}

#[pymethods]
impl DeviceConfigBuilderACPy {
    pub fn multicore(&self) -> DeviceConfigBuilderAMCPy {
        DeviceConfigBuilderAMCPy::new(self.inner.clone().multicore())
    }

    pub fn single(&self) -> DeviceConfigBuilderAMCPy {
        DeviceConfigBuilderAMCPy::new(self.inner.clone().single())
    }

    pub fn fused(&self) -> DeviceConfigBuilderAMCPy {
        DeviceConfigBuilderAMCPy::new(self.inner.clone().fused())
    }
}

#[pyclass(name = "DeviceConfigBuilderAMC")]
pub struct DeviceConfigBuilderAMCPy {
    inner: DeviceConfigBuilder<Arch, DeviceMode, u8>,
}

impl DeviceConfigBuilderAMCPy {
    pub fn new(d: DeviceConfigBuilder<Arch, DeviceMode, u8>) -> Self {
        Self { inner: d }
    }
}

#[pymethods]
impl DeviceConfigBuilderAMCPy {
    pub fn count(&self, count: u8) -> DeviceConfigPy {
        DeviceConfigPy::new(self.inner.clone().count(count))
    }

    pub fn build(&self) -> DeviceConfigPy {
        DeviceConfigPy::new(self.inner.clone().build())
    }
}
