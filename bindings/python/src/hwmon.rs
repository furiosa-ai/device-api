use std::sync::Arc;

use furiosa_device::hwmon::{Fetcher, SensorValue};
use pyo3::prelude::*;

use crate::errors::to_py_err;

#[pyclass(name = "SensorValue")]
pub struct SensorValuePy {
    #[pyo3(get)]
    label: String,
    #[pyo3(get)]
    value: i32,
}

impl SensorValuePy {
    pub fn new(s: &SensorValue) -> Self {
        Self {
            label: s.label.clone(),
            value: s.value,
        }
    }
}

#[pymethods]
impl SensorValuePy {
    fn __repr__(&self) -> String {
        format!("{}: {}", self.label, self.value)
    }
}

#[pyclass(name = "Fetcher", subclass)]
pub struct FetcherPy {
    pub inner: Arc<Fetcher>,
}

impl FetcherPy {
    pub fn new(f: &Fetcher) -> Self {
        Self {
            inner: Arc::new(f.clone()),
        }
    }
}

#[pymethods]
impl FetcherPy {
    fn read_currents<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let fetcher = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            fetcher
                .read_currents()
                .await
                .map(|vec| {
                    vec.iter()
                        .map(SensorValuePy::new)
                        .collect::<Vec<SensorValuePy>>()
                })
                .map_err(to_py_err)
        })
    }

    fn read_voltages<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let fetcher = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            fetcher
                .read_voltages()
                .await
                .map(|vec| {
                    vec.iter()
                        .map(SensorValuePy::new)
                        .collect::<Vec<SensorValuePy>>()
                })
                .map_err(to_py_err)
        })
    }

    fn read_powers_average<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let fetcher = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            fetcher
                .read_powers_average()
                .await
                .map(|vec| {
                    vec.iter()
                        .map(SensorValuePy::new)
                        .collect::<Vec<SensorValuePy>>()
                })
                .map_err(to_py_err)
        })
    }

    fn read_temperatures<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let fetcher = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            fetcher
                .read_temperatures()
                .await
                .map(|vec| {
                    vec.iter()
                        .map(SensorValuePy::new)
                        .collect::<Vec<SensorValuePy>>()
                })
                .map_err(to_py_err)
        })
    }
}
