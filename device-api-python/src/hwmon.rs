use std::sync::Arc;

use furiosa_device::hwmon::{Fetcher, SensorValue};
use pyo3::prelude::*;
use tokio::runtime::Runtime;

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

#[pyclass(name = "Fetcher")]
pub struct FetcherPy {
    inner: Arc<Fetcher>,
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
    pub fn read_currents<'py, 'a>(&'a self, py: Python<'py>) -> PyResult<&'py PyAny> {
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

    pub fn read_voltages<'py, 'a>(&'a self, py: Python<'py>) -> PyResult<&'py PyAny> {
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

    pub fn read_powers_average<'py, 'a>(&'a self, py: Python<'py>) -> PyResult<&'py PyAny> {
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

    pub fn read_temperatures<'py, 'a>(&'a self, py: Python<'py>) -> PyResult<&'py PyAny> {
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
