use std::str::FromStr;

use furiosa_device::{Arch, DeviceConfig};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyType;

use crate::arch::ArchPy;
use crate::errors::to_py_err;

/// Describes a required set of devices for `find_device_files`.
///
/// # Examples
/// ```python
/// from furiosa_device import Arch, DeviceConfig
///
/// # 1 core Warboy
/// config = DeviceConfig(arch=Arch.Warboy)
///
/// # 1 core Warboy x 2
/// config = DeviceConfig(arch=Arch.Warboy, count=2)
///
/// # Fused 2 cores RNGD x 2
/// config = DeviceConfig(arch=Arch.RNGD, cores=2, count=2)
/// ```
///
/// # Textual Representation
///
/// DeviceConfig supports textual representation, which is its equivalent string representation.
/// One can obtain the corresponding DeviceConfig from the textual representation
/// by using the from_str function.
///
/// ```python
/// from furiosa_device import DeviceConfig
///
/// config = DeviceConfig.from_env("SOME_ENV_KEY")
/// config = DeviceConfig.from_str("rngd:0:0-3,rngd:0:4-7") # get config directly from a string literal
/// ```
///
/// The rules for textual representation are as follows:
///
/// ```python
/// from furiosa_device import DeviceConfig
///
/// # Named configuration examples (using specific device names)
/// config = DeviceConfig.from_str("warboy:0:0") # warboy, npu0pe0
/// config = DeviceConfig.from_str("warboy:0:0-1") # warboy, npu0pe0-1
/// config = DeviceConfig.from_str("rngd:0:0-3") # rngd, npu0pe0-3
/// config = DeviceConfig.from_str("rngd:1:4-5") # rngd, npu1pe4-5
/// config = DeviceConfig.from_str("npu:0:0") # warboy, npu0pe0; "npu" is an alias for "warboy" for backward compatibility
///
/// # Unnamed configuration examples
/// config = DeviceConfig.from_str("warboy*2") # single pe x 2 (equivalent to "warboy(1)*2")
/// config = DeviceConfig.from_str("warboy(1)*2") # single pe x 2
/// config = DeviceConfig.from_str("warboy(2)*2") # 2-pe fusioned x 2
/// config = DeviceConfig.from_str("rngd(1)*2") # single pe x 2
/// config = DeviceConfig.from_str("rngd(4)*1") # 4-pe fusioned x 1
///
/// # Combining multiple comma-separated representation is also possible.
/// config = DeviceConfig.from_str("warboy:0:0-1,warboy:1:0-1")
/// config = DeviceConfig.from_str("warboy:0:0-1,warboy(2)*1") // One named 2-pe warboy (npu0pe0-1), and one anonmyous 2-pe warboy
/// ```
#[pyclass(name = "DeviceConfig")]
#[derive(Clone)]
pub struct DeviceConfigPy {
    pub inner: DeviceConfig,
}

impl DeviceConfigPy {
    fn new(d: DeviceConfig) -> Self {
        Self { inner: d }
    }
}

#[pymethods]
impl DeviceConfigPy {
    #[new]
    #[pyo3(signature = (arch=ArchPy::Warboy, cores=1, count=1))]
    fn py_new(arch: ArchPy, cores: u8, count: u8) -> PyResult<DeviceConfigPy> {
        let config = match arch {
            ArchPy::Warboy => DeviceConfig::warboy(),
            ArchPy::RNGD => DeviceConfig::rngd(),
        };
        if !Arch::from(arch.clone()).is_fusible_count(cores) {
            return Err(PyRuntimeError::new_err(format!(
                "Invalid core count: {} cores are not available for {:?}",
                cores, arch
            )));
        }
        Ok(DeviceConfigPy::new(config.cores(cores).count(count)))
    }

    fn __repr__(&self) -> String {
        self.inner.to_string()
    }

    #[classmethod]
    fn from_env(_cls: &PyType, key: &str) -> PyResult<DeviceConfigPy> {
        DeviceConfig::from_env(key)
            .build()
            .map(DeviceConfigPy::new)
            .map_err(to_py_err)
    }

    #[classmethod]
    fn from_str(_cls: &PyType, key: &str) -> PyResult<DeviceConfigPy> {
        DeviceConfig::from_str(key)
            .map(DeviceConfigPy::new)
            .map_err(to_py_err)
    }
}
