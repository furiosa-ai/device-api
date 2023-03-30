use furiosa_device::{DeviceError, DeviceResult};
use pyo3::{PyErr, PyResult, exceptions::PyRuntimeError};


pub fn to_py_result<T>(result: DeviceResult<T>) -> PyResult<T> {
    match result {
        Ok(r) => Ok(r),
        Err(e) => Err(to_py_err(e)),
    }
}

pub fn to_py_err(err: DeviceError) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}