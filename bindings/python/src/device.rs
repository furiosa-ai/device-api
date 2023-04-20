use std::collections::HashMap;
use std::sync::Arc;

use furiosa_device::{Arch, CoreRange, CoreStatus, Device, DeviceFile, DeviceMode};
use pyo3::prelude::*;

use crate::arch::ArchPy;
use crate::errors::to_py_err;
use crate::hwmon::FetcherPy;

#[pyclass(name = "CoreStatusType")]
#[derive(Clone)]
enum CoreStatusTypePy {
    Available,
    Occupied,
    Unavailable,
}

#[pyclass(name = "CoreStatus")]
#[derive(Clone)]
struct CoreStatusPy {
    #[pyo3(get)]
    status_type: CoreStatusTypePy,
    #[pyo3(get)]
    value: Option<String>,
}

impl CoreStatusPy {
    fn new(cs: CoreStatus) -> Self {
        match cs {
            CoreStatus::Available => Self {
                status_type: CoreStatusTypePy::Available,
                value: None,
            },
            CoreStatus::Occupied(s) => Self {
                status_type: CoreStatusTypePy::Occupied,
                value: Some(s),
            },
            CoreStatus::Unavailable => Self {
                status_type: CoreStatusTypePy::Unavailable,
                value: None,
            },
        }
    }
}

#[pyclass(name = "Device")]
pub struct DevicePy {
    inner: Arc<Device>,
}

impl DevicePy {
    pub fn new(dev: Device) -> Self {
        Self {
            inner: Arc::new(dev),
        }
    }
}

#[pymethods]
impl DevicePy {
    fn name(&self) -> String {
        self.inner.name()
    }

    fn device_index(&self) -> u8 {
        self.inner.device_index()
    }

    fn arch(&self) -> ArchPy {
        match self.inner.arch() {
            Arch::WarboyA0 => ArchPy::WarboyA0,
            Arch::WarboyB0 => ArchPy::Warboy,
            Arch::Renegade => ArchPy::Renegade,
            Arch::U250 => ArchPy::U250,
        }
    }

    fn alive(&self) -> PyResult<bool> {
        self.inner.alive().map_err(to_py_err)
    }

    fn atr_error(&self) -> PyResult<HashMap<String, u32>> {
        self.inner.atr_error().map_err(to_py_err)
    }

    fn busname(&self) -> PyResult<String> {
        self.inner.busname().map_err(to_py_err)
    }

    fn pci_dev(&self) -> PyResult<String> {
        self.inner.pci_dev().map_err(to_py_err)
    }

    fn device_sn(&self) -> PyResult<String> {
        self.inner.device_sn().map_err(to_py_err)
    }

    fn device_uuid(&self) -> PyResult<String> {
        self.inner.device_uuid().map_err(to_py_err)
    }

    fn firmware_version(&self) -> PyResult<String> {
        self.inner.firmware_version().map_err(to_py_err)
    }

    fn driver_version(&self) -> PyResult<String> {
        self.inner.driver_version().map_err(to_py_err)
    }

    fn heartbeat(&self) -> PyResult<u32> {
        self.inner.heartbeat().map_err(to_py_err)
    }

    fn ctrl_device_led(&self, led: (bool, bool, bool)) -> PyResult<()> {
        self.inner.ctrl_device_led(led).map_err(to_py_err)
    }

    fn core_num(&self) -> u8 {
        self.inner.core_num()
    }

    fn cores(&self) -> Vec<u8> {
        self.inner.cores().to_vec()
    }

    fn dev_files(&self) -> Vec<DeviceFilePy> {
        self.inner
            .dev_files()
            .iter()
            .map(|d| DeviceFilePy::new(d.clone()))
            .collect()
    }

    fn get_status_core<'py, 'a>(&'a self, py: Python<'py>, core: u8) -> PyResult<&'py PyAny> {
        let device = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            device
                .get_status_core(core)
                .await
                .map(CoreStatusPy::new)
                .map_err(to_py_err)
        })
    }

    fn get_status_all<'py, 'a>(&'a self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let device = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            device
                .get_status_all()
                .await
                .map(|r| {
                    r.into_iter()
                        .map(|(k, v)| (k, CoreStatusPy::new(v)))
                        .collect::<HashMap<u8, CoreStatusPy>>()
                })
                .map_err(to_py_err)
        })
    }

    pub fn get_hwmon_fetcher(&self) -> FetcherPy {
        FetcherPy::new(self.inner.get_hwmon_fetcher())
    }
}

#[pyclass(name = "CoreRangeType")]
#[derive(Clone)]
enum CoreRangeTypePy {
    All,
    Range,
}

#[pyclass(name = "CoreRange")]
#[derive(Clone)]
pub struct CoreRangePy {
    #[pyo3(get)]
    range_type: CoreRangeTypePy,
    #[pyo3(get)]
    value: Option<(u8, u8)>,
}

impl CoreRangePy {
    fn new(cr: CoreRange) -> Self {
        match cr {
            CoreRange::All => Self {
                range_type: CoreRangeTypePy::All,
                value: None,
            },
            CoreRange::Range(r) => Self {
                range_type: CoreRangeTypePy::Range,
                value: Some(r),
            },
        }
    }
}

#[pymethods]
impl CoreRangePy {
    fn contains(&self, idx: u8) -> bool {
        if let Some((s, e)) = self.value {
            (s..=e).contains(&idx)
        } else {
            true
        }
    }
}

#[pyclass(name = "DeviceFile")]
#[derive(Clone)]
pub struct DeviceFilePy {
    inner: DeviceFile,
}

impl DeviceFilePy {
    pub fn new(devf: DeviceFile) -> Self {
        Self { inner: devf }
    }
}

#[pymethods]
impl DeviceFilePy {
    fn path(&self) -> &str {
        self.inner.path().to_str().unwrap()
    }

    fn filename(&self) -> &str {
        self.inner.filename()
    }

    fn device_index(&self) -> u8 {
        self.inner.device_index()
    }

    fn core_range(&self) -> CoreRangePy {
        CoreRangePy::new(self.inner.core_range())
    }

    fn mode(&self) -> DeviceModePy {
        match self.inner.mode() {
            DeviceMode::Single => DeviceModePy::Single,
            DeviceMode::Fusion => DeviceModePy::Fusion,
            DeviceMode::MultiCore => DeviceModePy::MultiCore,
        }
    }
}

#[pyclass(name = "DeviceMode")]
#[derive(Clone)]
pub enum DeviceModePy {
    Single,
    Fusion,
    MultiCore,
}
