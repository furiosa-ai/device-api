use furiosa_device::hwmon::{Fetcher, SensorValue};
use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::errors::to_py_err;

#[pyclass(name = "SensorValue")]
pub struct SensorValuePy {
    label: String,
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
    inner: Fetcher,
}

impl FetcherPy {
    pub fn new(f: &Fetcher) -> Self {
        Self { inner: f.clone() }
    }
}

#[pymethods]
impl FetcherPy {
    pub fn read_currents(&self) -> PyResult<Vec<SensorValuePy>> {
        let currents = Runtime::new()
            .unwrap()
            .block_on(self.inner.read_currents())
            .map_err(to_py_err);
        match currents {
            Ok(c) => Ok(c.iter().map(SensorValuePy::new).collect()),
            Err(e) => Err(e),
        }
    }

    pub fn read_voltages(&self) -> PyResult<Vec<SensorValuePy>> {
        let voltages = Runtime::new()
            .unwrap()
            .block_on(self.inner.read_voltages())
            .map_err(to_py_err);
        match voltages {
            Ok(v) => Ok(v.iter().map(SensorValuePy::new).collect()),
            Err(e) => Err(e),
        }
    }

    pub fn read_powers_average(&self) -> PyResult<Vec<SensorValuePy>> {
        let powers = Runtime::new()
            .unwrap()
            .block_on(self.inner.read_powers_average())
            .map_err(to_py_err);
        match powers {
            Ok(p) => Ok(p.iter().map(SensorValuePy::new).collect()),
            Err(e) => Err(e),
        }
    }

    pub fn read_temperatures(&self) -> PyResult<Vec<SensorValuePy>> {
        let temperatures = Runtime::new()
            .unwrap()
            .block_on(self.inner.read_temperatures())
            .map_err(to_py_err);
        match temperatures {
            Ok(t) => Ok(t.iter().map(SensorValuePy::new).collect()),
            Err(e) => Err(e),
        }
    }
}
