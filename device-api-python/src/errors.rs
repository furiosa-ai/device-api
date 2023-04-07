use furiosa_device::DeviceError;
use pyo3::{exceptions::PyRuntimeError, PyErr};

pub fn to_py_err(err: DeviceError) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}
