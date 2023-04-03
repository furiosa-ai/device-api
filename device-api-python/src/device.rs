use std::collections::HashMap;
use std::sync::Arc;

use furiosa_device::{Arch, CoreRange, CoreStatus, Device, DeviceFile, DeviceMode};
use pyo3::prelude::*;

use crate::arch::ArchPy;
use crate::errors::to_py_err;
use crate::hwmon::FetcherPy;

#[pyclass(name = "Available")]
pub struct AvailablePy {}

impl AvailablePy {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for AvailablePy {
    fn default() -> Self {
        AvailablePy::new()
    }
}

#[pyclass(name = "Unavailable")]
pub struct UnavailablePy {}

impl UnavailablePy {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for UnavailablePy {
    fn default() -> Self {
        UnavailablePy::new()
    }
}

#[pyclass(name = "Occupied")]
pub struct OccupiedPy {
    str: String,
}

impl OccupiedPy {
    pub fn new(s: String) -> Self {
        Self { str: s }
    }
}

#[pyclass(name = "CoreStatus")]
pub struct CoreStatusPy {
    available: Option<AvailablePy>,
    occupied: Option<OccupiedPy>,
    unavailable: Option<UnavailablePy>,
}

impl CoreStatusPy {
    pub fn new(cs: CoreStatus) -> Self {
        match cs {
            CoreStatus::Available => Self {
                available: Some(AvailablePy::new()),
                occupied: None,
                unavailable: None,
            },
            CoreStatus::Occupied(s) => Self {
                available: None,
                occupied: Some(OccupiedPy::new(s)),
                unavailable: None,
            },
            CoreStatus::Unavailable => Self {
                available: None,
                occupied: None,
                unavailable: Some(UnavailablePy::new()),
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
    pub fn name(&self) -> String {
        self.inner.name()
    }

    pub fn device_index(&self) -> u8 {
        self.inner.device_index()
    }

    pub fn arch(&self) -> ArchPy {
        match self.inner.arch() {
            Arch::Warboy => ArchPy::Warboy,
            Arch::WarboyB0 => ArchPy::WarboyB0,
            Arch::Renegade => ArchPy::Renegade,
            Arch::U250 => ArchPy::U250,
        }
    }

    pub fn alive(&self) -> PyResult<bool> {
        self.inner.alive().map_err(to_py_err)
    }

    pub fn atr_error(&self) -> PyResult<HashMap<String, u32>> {
        self.inner.atr_error().map_err(to_py_err)
    }

    pub fn busname(&self) -> PyResult<String> {
        self.inner.busname().map_err(to_py_err)
    }

    pub fn pci_dev(&self) -> PyResult<String> {
        self.inner.pci_dev().map_err(to_py_err)
    }

    pub fn device_sn(&self) -> PyResult<String> {
        self.inner.device_sn().map_err(to_py_err)
    }

    pub fn device_uuid(&self) -> PyResult<String> {
        self.inner.device_uuid().map_err(to_py_err)
    }

    pub fn firmware_version(&self) -> PyResult<String> {
        self.inner.firmware_version().map_err(to_py_err)
    }

    pub fn hearbeat(&self) -> PyResult<u32> {
        self.inner.heartbeat().map_err(to_py_err)
    }

    pub fn ctrl_device_led(&self, led: (bool, bool, bool)) -> PyResult<()> {
        self.inner.ctrl_device_led(led).map_err(to_py_err)
    }

    pub fn core_num(&self) -> u8 {
        self.inner.core_num()
    }

    pub fn cores(&self) -> Vec<u8> {
        self.inner.cores().to_vec()
    }

    pub fn dev_files(&self) -> Vec<DeviceFilePy> {
        self.inner
            .dev_files()
            .iter()
            .map(|d| DeviceFilePy::new(d.clone()))
            .collect()
    }

    pub fn get_status_core<'py, 'a>(&'a self, py: Python<'py>, core: u8) -> PyResult<&'py PyAny> {
        let device = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            device
                .get_status_core(core)
                .await
                .map(CoreStatusPy::new)
                .map_err(to_py_err)
        })
    }

    pub fn get_status_all<'py, 'a>(&'a self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let device = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            device
                .get_status_all()
                .await
                .map(|r| {
                    r.iter()
                        .map(|(k, v)| (*k, CoreStatusPy::new(v.clone())))
                        .collect::<HashMap<u8, CoreStatusPy>>()
                })
                .map_err(to_py_err)
        })
    }

    pub fn get_hwmon_fetcher(&self) -> FetcherPy {
        FetcherPy::new(self.inner.get_hwmon_fetcher())
    }
}

// exporting enum with field to python is not supported yet
// https://github.com/PyO3/pyo3/issues/417
#[pyclass(name = "All")]
#[derive(Default, Clone)]
pub struct AllPy {}

impl AllPy {
    pub fn new() -> Self {
        Self {}
    }
}

#[pyclass(name = "Range")]
pub struct RangePy {
    range: (u8, u8),
}

impl RangePy {
    pub fn new(r: (u8, u8)) -> Self {
        Self { range: r }
    }
}

#[pyclass(name = "CoreRange")]
pub struct CoreRangePy {
    all: Option<AllPy>,
    range: Option<RangePy>,
}

impl CoreRangePy {
    pub fn new(cr: CoreRange) -> Self {
        match cr {
            CoreRange::All => Self {
                all: Some(AllPy::new()),
                range: None,
            },
            CoreRange::Range(r) => Self {
                all: None,
                range: Some(RangePy::new(r)),
            },
        }
    }
}

#[pymethods]
impl CoreRangePy {
    pub fn contains(&self, idx: u8) -> bool {
        match &self.range {
            Some(r) => {
                let (s, e) = r.range;
                (s..=e).contains(&idx)
            }
            None => true,
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
    pub fn path(&self) -> &str {
        self.inner.path().to_str().unwrap()
    }

    pub fn filename(&self) -> &str {
        self.inner.filename()
    }

    pub fn device_index(&self) -> u8 {
        self.inner.device_index()
    }

    pub fn core_range(&self) -> CoreRangePy {
        CoreRangePy::new(self.inner.core_range())
    }

    pub fn mode(&self) -> DeviceModePy {
        match self.inner.mode() {
            DeviceMode::Single => DeviceModePy::Single,
            DeviceMode::Fusion => DeviceModePy::Fusion,
            DeviceMode::MultiCore => DeviceModePy::MultiCore,
        }
    }
}

#[pyclass(name = "DeviceMode")]
pub enum DeviceModePy {
    Single,
    Fusion,
    MultiCore,
}
