use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::path::PathBuf;

use dyn_clone::DynClone;
use lazy_static::lazy_static;
use regex::Regex;

use crate::arch::Arch;
use crate::hwmon;
use crate::perf_regs::PerformanceCounter;
use crate::status::{get_device_status, DeviceStatus};
use crate::sysfs::{npu_mgmt, pci};
use crate::{devfs, DeviceError, DeviceResult};

#[derive(Debug, Clone)]

/// Abstraction for a single Furiosa NPU device.
///
/// # About Furiosa NPU
///
/// A Furiosa NPU device contains a number of cores and offers several ways called
/// [`DeviceMode`][crate::DeviceMode] to combine multiple cores to a single logical device,
/// as following:
/// * [`Single`][crate::DeviceMode::Single]: A logical device is composed of a single core.
/// * [`Fusion`][crate::DeviceMode::Fusion]: Multiple cores work together as if
///     they were one device. This mode is useful when a DNN model requires
///      much computation power and large memory capacity.
/// * [`MultiCore`][crate::DeviceMode::MultiCore]: A logical device uses multiple cores,
///     each of which communicates to one another through interconnect.
///     In this mode, partitions of a model or multiple models can be pipelined.
/// (See [`DeviceConfig`][crate::DeviceConfig] and
/// [`find_device_files`][crate::find_device_files]).
///
/// Hence a Furiosa NPU device exposes several devfs files for each purpose
/// above. They can be listed by calling [`dev_files`][Device::dev_files]
/// method, which returns a list of [`DeviceFile`]s.
/// Each [`DeviceFile`] again offers [`mode`][DeviceFile::mode] method to
/// identify its [`DeviceMode`].
pub struct Device {
    inner: Box<dyn DeviceInner>,
    hwmon_fetcher: hwmon::Fetcher,
    pub(crate) cores: Vec<CoreIdx>,
    pub(crate) dev_files: Vec<DeviceFile>,
}

pub(crate) trait DeviceInner:
    DeviceMgmt + DeviceCtrl + DevicePerf + DynClone + Send + Sync + Debug
{
}

dyn_clone::clone_trait_object!(DeviceInner);

pub(crate) trait DeviceMgmt {
    fn sysfs(&self) -> &PathBuf;
    fn arch(&self) -> Arch;
    fn devfile_index(&self) -> u8;
    fn name(&self) -> String;
    fn busname(&self) -> String;
    fn pci_dev(&self) -> String;
    fn device_sn(&self) -> String;
    fn device_uuid(&self) -> String;
    fn firmware_version(&self) -> String;
    fn driver_version(&self) -> String;
    fn alive(&self) -> DeviceResult<bool>;
    fn atr_error(&self) -> DeviceResult<HashMap<String, u32>>;
    fn heartbeat(&self) -> DeviceResult<u32>;
    fn clock_frequency(&self) -> DeviceResult<Vec<ClockFrequency>>;
}

pub(crate) trait DeviceCtrl {
    fn ctrl_device_led(&self, led: (bool, bool, bool)) -> DeviceResult<()>;
    fn ctrl_ne_dtm_policy(&self, policy: npu_mgmt::DtmPolicy) -> DeviceResult<()>;
    fn ctrl_performance_level(&self, level: npu_mgmt::PerfLevel) -> DeviceResult<()>;
    fn ctrl_performance_mode(&self, mode: npu_mgmt::PerfMode) -> DeviceResult<()>;
}

pub(crate) trait DevicePerf {
    fn get_performance_counter(&self, file: &DeviceFile) -> DeviceResult<PerformanceCounter>;
}

impl Device {
    pub(crate) fn new(
        inner: Box<dyn DeviceInner>,
        hwmon_fetcher: hwmon::Fetcher,
        cores: Vec<CoreIdx>,
        dev_files: Vec<DeviceFile>,
    ) -> Self {
        Self {
            inner,
            hwmon_fetcher,
            cores,
            dev_files,
        }
    }

    /// Returns the name of the device, represented by the common prefix of the character device file (e.g., /dev/npu0).
    pub fn name(&self) -> String {
        self.inner.name()
    }

    /// Returns the device file index (e.g., 0 for /dev/npu0).
    pub fn devfile_index(&self) -> u8 {
        self.inner.devfile_index()
    }

    /// Returns `Arch` of the device(e.g., `Warboy`).
    pub fn arch(&self) -> Arch {
        self.inner.arch()
    }

    /// Returns a liveness state of the device.
    pub fn alive(&self) -> DeviceResult<bool> {
        self.inner.alive()
    }

    /// Returns error states of the device.
    pub fn atr_error(&self) -> DeviceResult<HashMap<String, u32>> {
        self.inner.atr_error()
    }

    /// Returns PCI bus number of the device.
    pub fn busname(&self) -> String {
        self.inner.busname()
    }

    /// Returns PCI device ID of the device.
    pub fn pci_dev(&self) -> String {
        self.inner.pci_dev()
    }

    /// Returns serial number of the device.
    pub fn device_sn(&self) -> String {
        self.inner.device_sn()
    }

    /// Returns UUID of the device.
    pub fn device_uuid(&self) -> String {
        self.inner.device_uuid()
    }

    /// Retrieves firmware revision from the device.
    pub fn firmware_version(&self) -> String {
        self.inner.firmware_version()
    }

    /// Retrieves driver version for the device.
    pub fn driver_version(&self) -> String {
        self.inner.driver_version()
    }

    /// Returns uptime of the device.
    pub fn heartbeat(&self) -> DeviceResult<u32> {
        self.inner.heartbeat()
    }

    /// Returns clock frequencies of components in the device.
    pub fn clock_frequency(&self) -> DeviceResult<Vec<ClockFrequency>> {
        self.inner.clock_frequency()
    }

    /// Controls the device led.
    #[allow(dead_code)]
    fn ctrl_device_led(&self, led: (bool, bool, bool)) -> DeviceResult<()> {
        self.inner.ctrl_device_led(led)
    }

    /// Control NE clocks.
    #[allow(dead_code)]
    fn ctrl_ne_clock(&self, _toggle: npu_mgmt::Toggle) -> DeviceResult<()> {
        unimplemented!()
    }

    /// Control the Dynamic Thermal Management policy.
    #[allow(dead_code)]
    fn ctrl_ne_dtm_policy(&self, policy: npu_mgmt::DtmPolicy) -> DeviceResult<()> {
        self.inner.ctrl_ne_dtm_policy(policy)
    }

    /// Control NE performance level
    #[allow(dead_code)]
    fn ctrl_performance_level(&self, level: npu_mgmt::PerfLevel) -> DeviceResult<()> {
        self.inner.ctrl_performance_level(level)
    }

    /// Control NE performance mode
    #[allow(dead_code)]
    fn ctrl_performance_mode(&self, mode: npu_mgmt::PerfMode) -> DeviceResult<()> {
        self.inner.ctrl_performance_mode(mode)
    }

    /// Retrieve NUMA node ID associated with the NPU's PCI lane
    // XXX(n0gu): warboy and renegade share the same implementation, but this may change in the future devices.
    pub fn numa_node(&self) -> DeviceResult<NumaNode> {
        let busname = self.inner.busname();
        let id = pci::numa::read_numa_node(self.inner.sysfs(), &busname)?
            .parse::<i32>()
            .map_err(|e| {
                DeviceError::unexpected_value(format!("Unexpected numa node id: {}", e))
            })?;

        let node = match id {
            _ if id >= 0 => NumaNode::Id(id as usize),
            _ if id == -1 => NumaNode::UnSupported,
            _ => {
                return Err(DeviceError::unexpected_value(format!(
                    "Unexpected numa node id: {id}"
                )))
            }
        };

        // TODO(n0gu): cache result
        Ok(node)
    }

    /// Counts the number of cores.
    pub fn core_num(&self) -> u8 {
        u8::try_from(self.cores.len()).unwrap()
    }

    /// List the core indices.
    pub fn cores(&self) -> &Vec<CoreIdx> {
        &self.cores
    }

    /// List device files under this device.
    pub fn dev_files(&self) -> &Vec<DeviceFile> {
        &self.dev_files
    }

    /// List performance counters for each device files.
    pub fn performance_counters(&self) -> Vec<(&DeviceFile, PerformanceCounter)> {
        let mut counters = vec![];

        for dev_file in self.dev_files() {
            if let Ok(perf_counter) = self.inner.get_performance_counter(dev_file) {
                counters.push((dev_file, perf_counter));
            }
        }

        counters
    }

    /// Examine a specific core of the device, whether it is available or not.
    pub async fn get_status_core(&self, core: CoreIdx) -> DeviceResult<CoreStatus> {
        for file in &self.dev_files {
            // get status of the exact core
            if file.mode() != DeviceMode::Single {
                continue;
            }
            if (file.core_range().contains(&core))
                && get_device_status(&file.path).await? == DeviceStatus::Occupied
            {
                return Ok(CoreStatus::Occupied(file.to_string()));
            }
        }
        Ok(CoreStatus::Available)
    }

    /// Examine each core of the device, whether it is available or not.
    pub async fn get_status_all(&self) -> DeviceResult<HashMap<CoreIdx, CoreStatus>> {
        let mut status_map = self.new_status_map();

        for core in self.cores() {
            let status = self.get_status_core(*core).await?;
            status_map.insert(*core, status);
        }
        Ok(status_map)
    }

    pub(crate) fn new_status_map(&self) -> HashMap<CoreIdx, CoreStatus> {
        self.cores
            .iter()
            .map(|k| (*k, CoreStatus::Available))
            .collect()
    }

    /// Returns `Fetcher` for hwmon metric of the device.
    pub fn get_hwmon_fetcher(&self) -> &hwmon::Fetcher {
        &self.hwmon_fetcher
    }
}

impl Display for Device {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Eq for Device {}

impl Ord for Device {
    fn cmp(&self, other: &Self) -> Ordering {
        self.devfile_index().cmp(&other.devfile_index())
    }
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.inner.devfile_index() == other.inner.devfile_index()
            && self.inner.arch() == other.inner.arch()
            && self.hwmon_fetcher == other.hwmon_fetcher
            && self.cores == other.cores
            && self.dev_files == other.dev_files
    }
}

impl PartialOrd for Device {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Non Uniform Memory Access (NUMA) node
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum NumaNode {
    UnSupported,
    Id(usize),
}

/// Enum for NPU core status.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CoreStatus {
    Available,
    Occupied(String),
    Unavailable,
}

impl Display for CoreStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CoreStatus::Available => write!(f, "available"),
            CoreStatus::Occupied(devfile) => write!(f, "occupied by {devfile}"),
            CoreStatus::Unavailable => write!(f, "unavailable"),
        }
    }
}

pub(crate) type CoreIdx = u8;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum CoreRange {
    All, // TODO: rename this to MultiCore
    // This range is inclusive [s, e]
    Range((u8, u8)),
}

impl CoreRange {
    pub fn contains(&self, idx: &CoreIdx) -> bool {
        match self {
            CoreRange::All => true,
            CoreRange::Range((s, e)) => (*s..=*e).contains(idx),
        }
    }

    pub fn has_intersection(&self, other: &Self) -> bool {
        match (self, other) {
            (CoreRange::All, _) | (_, CoreRange::All) => true,
            (CoreRange::Range(a), CoreRange::Range(b)) => !(a.1 < b.0 || b.1 < a.0),
        }
    }
}

impl Ord for CoreRange {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            CoreRange::All => {
                if self == other {
                    std::cmp::Ordering::Equal
                } else {
                    std::cmp::Ordering::Less
                }
            }
            CoreRange::Range(r) => match other {
                CoreRange::All => std::cmp::Ordering::Greater,
                CoreRange::Range(other) => (r.1 - r.0).cmp(&(other.1 - other.0)).then(r.cmp(other)),
            },
        }
    }
}

impl PartialOrd for CoreRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<u8> for CoreRange {
    fn from(id: u8) -> Self {
        Self::Range((id, id))
    }
}

impl TryFrom<(u8, u8)> for CoreRange {
    type Error = ();
    fn try_from(v: (u8, u8)) -> Result<Self, Self::Error> {
        if v.0 < v.1 {
            Ok(Self::Range(v))
        } else {
            Err(())
        }
    }
}

/// An abstraction for a device file and its mode.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct DeviceFile {
    pub(crate) devfile_index: u8,
    pub(crate) core_range: CoreRange,
    pub(crate) path: PathBuf,
    pub(crate) mode: DeviceMode,
}

impl Display for DeviceFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.file_name().unwrap().to_str().unwrap())
    }
}

impl Ord for DeviceFile {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.devfile_index, self.core_range).cmp(&(other.devfile_index(), other.core_range()))
    }
}

impl PartialOrd for DeviceFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl DeviceFile {
    /// Returns `PathBuf` to the device file.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns the file name (e.g., npu0pe0 for /dev/npu0pe0).
    pub fn filename(&self) -> &str {
        // We should guarantee that it returns a filename
        self.path
            .file_name()
            .expect("not a file")
            .to_str()
            .expect("invalid UTF-8 encoding")
    }

    /// Returns the device index (e.g., 1 for npu1pe0).
    pub fn devfile_index(&self) -> u8 {
        self.devfile_index
    }

    /// Returns the range of cores this device file may occupy.
    pub fn core_range(&self) -> CoreRange {
        self.core_range
    }

    /// Return the mode of this device file.
    pub fn mode(&self) -> DeviceMode {
        self.mode
    }

    pub fn has_intersection(&self, other: &Self) -> bool {
        self.devfile_index() == other.devfile_index()
            && self.core_range().has_intersection(&other.core_range())
    }
}

impl TryFrom<&PathBuf> for DeviceFile {
    type Error = DeviceError;

    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let file_name = path
            .file_name()
            .expect("not a file")
            .to_string_lossy()
            .to_string();

        let (devfile_index, core_indices) = devfs::parse_indices(file_name)?;

        let (mode, core_range) = match core_indices.len() {
            0 => (DeviceMode::MultiCore, CoreRange::All),
            1 => (DeviceMode::Single, CoreRange::from(core_indices[0])),
            n => (
                DeviceMode::Fusion,
                CoreRange::try_from((core_indices[0], core_indices[n - 1]))
                    .map_err(|_| DeviceError::unrecognized_file(path.to_string_lossy()))?,
            ),
        };

        Ok(DeviceFile {
            devfile_index,
            core_range,
            path: path.clone(),
            mode,
        })
    }
}

/// Enum for NPU's operating mode.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, enum_utils::FromStr, PartialOrd)]
#[enumeration(case_insensitive)]
pub enum DeviceMode {
    Single,
    Fusion,
    MultiCore,
}

lazy_static! {
    // Update CLOCK_FREQUENCY_FMT when you change this pattern
    static ref CLOCK_FREQUENCY_FMT: Regex =
    Regex::new(r"(?P<name>(\w| )+)\((?P<unit>.*)\): (?P<value>\d+)").unwrap();
}

#[derive(Clone)]
pub struct ClockFrequency {
    pub(crate) name: String,
    pub(crate) unit: String,
    pub(crate) value: u32,
}

impl TryFrom<&str> for ClockFrequency {
    type Error = ();

    fn try_from(line: &str) -> Result<Self, Self::Error> {
        let items = CLOCK_FREQUENCY_FMT.captures(line).ok_or(())?;
        let name = items.name("name").ok_or(())?.as_str().trim();
        let unit = items.name("unit").ok_or(())?.as_str().trim();
        let value = items.name("value").ok_or(())?.as_str().trim();

        Ok(Self {
            name: name.to_string(),
            unit: unit.to_string(),
            value: value.parse().map_err(|_| ())?,
        })
    }
}

impl ClockFrequency {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn unit(&self) -> &str {
        self.unit.as_str()
    }

    pub fn value(&self) -> u32 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_range_ordering() {
        let all = CoreRange::All;
        let core0 = CoreRange::Range((0, 0));
        let core1 = CoreRange::Range((1, 1));
        let core0_1 = CoreRange::Range((0, 1));
        let core0_3 = CoreRange::Range((0, 3));
        let core2_3 = CoreRange::Range((2, 3));

        assert!(all < core0);
        assert!(core0 < core1);
        assert!(core1 < core0_1);
        assert!(core0_1 < core2_3);
        assert!(core2_3 < core0_3);
    }

    #[test]
    fn test_try_from() -> Result<(), DeviceError> {
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0"))?,
            DeviceFile {
                devfile_index: 0,
                path: PathBuf::from("./npu0"),
                core_range: CoreRange::All,
                mode: DeviceMode::MultiCore,
            }
        );
        assert!(DeviceFile::try_from(&PathBuf::from("./npu0pe")).is_err());
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0pe0"))?,
            DeviceFile {
                devfile_index: 0,
                path: PathBuf::from("./npu0pe0"),
                core_range: CoreRange::Range((0, 0)),
                mode: DeviceMode::Single,
            }
        );
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0pe1"))?,
            DeviceFile {
                devfile_index: 0,
                path: PathBuf::from("./npu0pe1"),
                core_range: CoreRange::Range((1, 1)),
                mode: DeviceMode::Single,
            }
        );
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0pe0-1"))?,
            DeviceFile {
                devfile_index: 0,
                path: PathBuf::from("./npu0pe0-1"),
                core_range: CoreRange::Range((0, 1)),
                mode: DeviceMode::Fusion,
            }
        );
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0pe0-2"))?,
            DeviceFile {
                devfile_index: 0,
                path: PathBuf::from("./npu0pe0-2"),
                core_range: CoreRange::Range((0, 2)),
                mode: DeviceMode::Fusion,
            }
        );
        assert!(DeviceFile::try_from(&PathBuf::from("./npu0pe0-")).is_err());
        assert!(DeviceFile::try_from(&PathBuf::from("./npu0pe-1")).is_err());
        Ok(())
    }

    #[test]
    fn test_core_status_fmt() {
        assert_eq!(format!("{}", CoreStatus::Available), "available");
        assert_eq!(format!("{}", CoreStatus::Unavailable), "unavailable");
        assert_eq!(
            format!("{}", CoreStatus::Occupied(String::from("npu0pe0"))),
            "occupied by npu0pe0"
        );
    }

    #[test]
    fn test_device_mode_from_str() {
        assert_eq!("single".parse(), Ok(DeviceMode::Single));
        assert_eq!("SiNgLe".parse(), Ok(DeviceMode::Single));
        assert_eq!("fusion".parse(), Ok(DeviceMode::Fusion));
        assert_eq!("fUsIoN".parse(), Ok(DeviceMode::Fusion));
        assert_eq!("multicore".parse(), Ok(DeviceMode::MultiCore));
        assert_eq!("MultiCore".parse(), Ok(DeviceMode::MultiCore));
        assert_eq!("invalid".parse::<DeviceMode>(), Err(()));
    }

    #[test]
    fn test_clock_frequency() {
        let line = "ne tensor (MHz): 2000";
        let res = ClockFrequency::try_from(line);
        assert!(res.is_ok());

        let res = res.unwrap();
        assert_eq!(res.name(), "ne tensor");
        assert_eq!(res.unit(), "MHz");
        assert_eq!(res.value(), 2000);
    }
}
