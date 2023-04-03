use furiosa_device::{DeviceConfig, EnvBuilder, NotDetermined};
use pyo3::prelude::*;

use super::DeviceConfigPy;
use crate::errors::to_py_err;

#[pyclass(name = "EnvBuilderNotDetermined")]
pub struct EnvBuilderNotDeterminedPy {
    inner: EnvBuilder<NotDetermined>,
}

impl EnvBuilderNotDeterminedPy {
    pub fn new(e: EnvBuilder<NotDetermined>) -> Self {
        Self { inner: e }
    }
}

#[pymethods]
impl EnvBuilderNotDeterminedPy {
    pub fn or_env(&self, key: String) -> Self {
        Self {
            inner: self.inner.clone().or_env(key),
        }
    }

    pub fn or_try(&self, key: Option<String>) -> Self {
        Self {
            inner: self.inner.clone().or_try(key),
        }
    }

    pub fn or(&self, fallback: DeviceConfigPy) -> EnvBuilderDevConfigPy {
        EnvBuilderDevConfigPy {
            inner: self.inner.clone().or(fallback.inner),
        }
    }

    pub fn or_default(&self) -> EnvBuilderDevConfigPy {
        EnvBuilderDevConfigPy {
            inner: self.inner.clone().or_default(),
        }
    }
}

#[pyclass(name = "EnvBuilderDevConfig")]
pub struct EnvBuilderDevConfigPy {
    inner: EnvBuilder<DeviceConfig>,
}

impl EnvBuilderDevConfigPy {
    pub fn new(e: EnvBuilder<DeviceConfig>) -> Self {
        Self { inner: e }
    }
}

#[pymethods]
impl EnvBuilderDevConfigPy {
    pub fn build(&self) -> PyResult<DeviceConfigPy> {
        self.inner
            .clone()
            .build()
            .map(DeviceConfigPy::new)
            .map_err(to_py_err)
    }
}
