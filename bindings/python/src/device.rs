use std::collections::HashMap;
use std::sync::Arc;

use furiosa_device::perf_regs::{PerformanceCounter, Utilization};
use furiosa_device::{
    Arch, ClockFrequency, CoreRange, CoreStatus, Device, DeviceFile, DeviceMode, NumaNode,
};
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

/// Enum for NPU core status.
#[pyclass(name = "CoreStatus")]
#[derive(Clone)]
pub struct CoreStatusPy {
    #[pyo3(get)]
    status_type: CoreStatusTypePy,
    #[pyo3(get)]
    value: Option<String>,
}

impl CoreStatusPy {
    pub fn new(cs: CoreStatus) -> Self {
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

#[pymethods]
impl CoreStatusPy {
    fn __repr__(&self) -> String {
        match self.status_type {
            CoreStatusTypePy::Available => String::from("Available"),
            CoreStatusTypePy::Occupied => format!("Occupied by {}", self.value.as_ref().unwrap()),
            CoreStatusTypePy::Unavailable => String::from("Unavailable"),
        }
    }
}

/// clock frequency of NPU components.
#[pyclass(name = "NeClockFrequency")]
#[derive(Clone)]
pub struct ClockFrequencyPy {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    unit: String,
    #[pyo3(get)]
    value: u32,
}

impl ClockFrequencyPy {
    pub fn new(cf: ClockFrequency) -> Self {
        Self {
            name: cf.name().to_string(),
            unit: cf.unit().to_string(),
            value: cf.value(),
        }
    }
}

#[pymethods]
impl ClockFrequencyPy {
    fn __repr__(&self) -> String {
        format!("{:15} : {} {}", self.name, self.value, self.unit)
    }
}

/// Abstraction for a single Furiosa NPU device.
///
/// # About Furiosa NPU
///
/// A Furiosa NPU device contains a number of cores and offers several ways called
/// `DeviceMode` to combine multiple cores to a single logical device,
/// as following:
/// * `Single`: A logical device is composed of a single core.
/// * `Fusion`: Multiple cores work together as if
///     they were one device. This mode is useful when a DNN model requires
///      much computation power and large memory capacity.
/// * `MultiCore`: A logical device uses multiple cores,
///     each of which communicates to one another through interconnect.
///     In this mode, partitions of a model or multiple models can be pipelined.
/// (See `DeviceConfig` and `find_device_files`).
///
/// Hence a Furiosa NPU device exposes several devfs files for each purpose
/// above. They can be listed by calling `dev_files` method, which returns a list of
/// `DeviceFile`s. Each `DeviceFile` again offers `mode` method to identify its
/// `DeviceMode`.
#[pyclass(name = "Device", subclass)]
pub struct DevicePy {
    pub inner: Arc<Device>,
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
    fn __repr__(&self) -> String {
        self.inner.to_string()
    }

    /// Return the name of the device (e.g., npu0).
    fn name(&self) -> String {
        self.inner.name()
    }

    /// Returns the device file index (e.g., 0 for npu0).
    fn devfile_index(&self) -> u8 {
        self.inner.devfile_index()
    }

    /// Returns `Arch` of the device(e.g., `Warboy`).
    fn arch(&self) -> ArchPy {
        match self.inner.arch() {
            Arch::WarboyB0 => ArchPy::Warboy,
            Arch::Renegade => ArchPy::Renegade,
        }
    }

    /// Returns a liveness state of the device.
    fn alive(&self) -> PyResult<bool> {
        self.inner.alive().map_err(to_py_err)
    }

    /// Returns error states of the device.
    fn atr_error(&self) -> PyResult<HashMap<String, u32>> {
        self.inner.atr_error().map_err(to_py_err)
    }

    /// Returns PCI bus number of the device.
    fn busname(&self) -> String {
        self.inner.busname()
    }

    /// Returns PCI device ID of the device.
    fn pci_dev(&self) -> String {
        self.inner.pci_dev()
    }

    /// Returns serial number of the device.
    fn device_sn(&self) -> String {
        self.inner.device_sn()
    }

    /// Returns UUID of the device.
    fn device_uuid(&self) -> String {
        self.inner.device_uuid()
    }

    /// Retrieves firmware revision from the device.
    fn firmware_version(&self) -> String {
        self.inner.firmware_version()
    }

    /// Retrieves driver version for the device.
    fn driver_version(&self) -> String {
        self.inner.driver_version()
    }

    /// Returns uptime of the device.
    fn heartbeat(&self) -> PyResult<u32> {
        self.inner.heartbeat().map_err(to_py_err)
    }

    /// Returns clock frequencies of components in the device.
    fn clock_frequency(&self) -> PyResult<Vec<ClockFrequencyPy>> {
        self.inner
            .clock_frequency()
            .map_err(to_py_err)
            .map(|v| v.into_iter().map(ClockFrequencyPy::new).collect())
    }

    /// Retrieve NUMA node ID associated with the NPU's PCI lane (-1 indicates unsupported)
    fn numa_node(&self) -> PyResult<i64> {
        self.inner
            .numa_node()
            .map(|n| match n {
                NumaNode::UnSupported => -1,
                NumaNode::Id(id) => id as i64,
            })
            .map_err(to_py_err)
    }

    /// Counts the number of cores.
    fn core_num(&self) -> u8 {
        self.inner.core_num()
    }

    /// List the core indices.
    fn cores(&self) -> Vec<u8> {
        self.inner.cores().to_vec()
    }

    /// List device files under this device.
    pub fn dev_files(&self) -> Vec<DeviceFilePy> {
        self.inner
            .dev_files()
            .iter()
            .cloned()
            .map(DeviceFilePy::new)
            .collect()
    }

    /// Retrieves the pair of device files and performance counters under this device.
    pub fn performance_counters(&self) -> Vec<(DeviceFilePy, PerformanceCounterPy)> {
        self.inner
            .performance_counters()
            .into_iter()
            .map(|(devfile, pc)| {
                (
                    DeviceFilePy::new(devfile.clone()),
                    PerformanceCounterPy::new(pc),
                )
            })
            .collect()
    }

    /// Examine a specific core of the device, whether it is available or not.
    fn get_status_core<'py>(&self, py: Python<'py>, core: u8) -> PyResult<&'py PyAny> {
        let device = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            device
                .get_status_core(core)
                .await
                .map(CoreStatusPy::new)
                .map_err(to_py_err)
        })
    }

    /// Examine each core of the device, whether it is available or not.
    fn get_status_all<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let device = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            device
                .get_status_all()
                .await
                .map(|map| {
                    map.into_iter()
                        .map(|(k, v)| (k, CoreStatusPy::new(v)))
                        .collect::<HashMap<u8, CoreStatusPy>>()
                })
                .map_err(to_py_err)
        })
    }

    /// Returns `Fetcher` for hwmon metric of the device.
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
    fn __repr__(&self) -> String {
        match self.range_type {
            CoreRangeTypePy::All => String::from("All"),
            CoreRangeTypePy::Range => format!(
                "Range ({}, {})",
                self.value.unwrap().0,
                self.value.unwrap().1
            ),
        }
    }

    pub fn contains(&self, idx: u8) -> bool {
        if let Some((s, e)) = self.value {
            (s..=e).contains(&idx)
        } else {
            true
        }
    }
}

/// An abstraction for a device file and its mode.
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
    fn __repr__(&self) -> String {
        self.inner.to_string()
    }

    /// Returns `PathBuf` to the device file.
    fn path(&self) -> &str {
        self.inner.path().to_str().unwrap()
    }

    /// Returns the file name (e.g., npu0pe0 for /dev/npu0pe0).
    fn filename(&self) -> &str {
        self.inner.filename()
    }

    /// Returns the device index (e.g., 1 for npu1pe0).
    fn devfile_index(&self) -> u8 {
        self.inner.devfile_index()
    }

    /// Returns the range of cores this device file may occupy.
    pub fn core_range(&self) -> CoreRangePy {
        CoreRangePy::new(self.inner.core_range())
    }

    /// Return the mode of this device file.
    fn mode(&self) -> DeviceModePy {
        match self.inner.mode() {
            DeviceMode::Single => DeviceModePy::Single,
            DeviceMode::Fusion => DeviceModePy::Fusion,
            DeviceMode::MultiCore => DeviceModePy::MultiCore,
        }
    }
}

/// Enum for NPU's operating mode.
#[pyclass(name = "DeviceMode")]
#[derive(Clone)]
pub enum DeviceModePy {
    Single,
    Fusion,
    MultiCore,
}

/// An abstraction for a performance counter.
#[pyclass(name = "PerformanceCounter")]
#[derive(Clone)]
pub struct PerformanceCounterPy {
    inner: PerformanceCounter,
}

impl PerformanceCounterPy {
    pub fn new(pc: PerformanceCounter) -> Self {
        Self { inner: pc }
    }
}

#[pymethods]
impl PerformanceCounterPy {
    fn __repr__(&self) -> String {
        format!(
            "PerformanceCounter {} {} {}",
            self.inner.cycle_count(),
            self.inner.task_execution_cycle(),
            self.inner.tensor_execution_cycle()
        )
    }

    /// Returns cycle count of the device file.
    pub fn cycle_count(&self) -> usize {
        self.inner.cycle_count()
    }

    /// Returns task execution cycle count of the device file.
    pub fn task_execution_cycle(&self) -> u32 {
        self.inner.task_execution_cycle()
    }

    /// Returns tensor execution cycle count of the device file.
    pub fn tensor_execution_cycle(&self) -> u32 {
        self.inner.tensor_execution_cycle()
    }

    /// Returns the difference between two counters.
    pub fn calculate_increased(&self, other: &PerformanceCounterPy) -> PerformanceCounterPy {
        PerformanceCounterPy::new(self.inner.calculate_increased(&other.inner))
    }

    /// Returns NPU utilization based on the difference between two counters.
    pub fn calculate_utilization(&self, other: &PerformanceCounterPy) -> UtilizationPy {
        UtilizationPy::new(self.inner.calculate_utilization(&other.inner))
    }
}

/// An abstraction for a utilization.
#[pyclass(name = "Utilization")]
#[derive(Clone)]
pub struct UtilizationPy {
    inner: Utilization,
}

impl UtilizationPy {
    pub fn new(util: Utilization) -> Self {
        Self { inner: util }
    }
}

#[pymethods]
impl UtilizationPy {
    fn __repr__(&self) -> String {
        format!("NPU Utilization {}", self.inner.npu_utilization())
    }

    pub fn npu_utilization(&self) -> f64 {
        self.inner.npu_utilization()
    }

    pub fn computation_ratio(&self) -> f64 {
        self.inner.computation_ratio()
    }

    pub fn io_ratio(&self) -> f64 {
        self.inner.io_ratio()
    }
}
