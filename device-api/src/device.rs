use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;

use lazy_static::lazy_static;
use regex::Regex;
use strum::IntoEnumIterator;

use crate::arch::Arch;
use crate::hwmon;
use crate::perf_regs::PerformanceCounter;
use crate::status::{get_device_status, DeviceStatus};
use crate::sysfs::npu_mgmt::{self, *};
use crate::sysfs::pci;
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
    device_info: DeviceInfo,
    hwmon_fetcher: hwmon::Fetcher,
    pub(crate) cores: Vec<CoreIdx>,
    pub(crate) dev_files: Vec<DeviceFile>,
}

impl Device {
    pub(crate) fn new(
        device_info: DeviceInfo,
        hwmon_fetcher: hwmon::Fetcher,
        cores: Vec<CoreIdx>,
        dev_files: Vec<DeviceFile>,
    ) -> Self {
        Self {
            device_info,
            hwmon_fetcher,
            cores,
            dev_files,
        }
    }

    /// Returns the name of the device (e.g., npu0).
    pub fn name(&self) -> String {
        format!("npu{}", self.device_index())
    }

    /// Returns the device index (e.g., 0 for npu0).
    pub fn device_index(&self) -> u8 {
        self.device_info.device_index
    }

    /// Returns the `DeviceInfo` struct.
    fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    /// Returns `Arch` of the device(e.g., `Warboy`).
    pub fn arch(&self) -> Arch {
        self.device_info().arch()
    }

    /// Returns a liveness state of the device.
    pub fn alive(&self) -> DeviceResult<bool> {
        self.device_info.get(&DynamicMgmtFile::Alive).and_then(|v| {
            npu_mgmt::parse_zero_or_one_to_bool(&v).ok_or_else(|| {
                DeviceError::unexpected_value(format!(
                    "Bad alive value: {v} (only 0 or 1 expected)"
                ))
            })
        })
    }

    /// Returns error states of the device.
    pub fn atr_error(&self) -> DeviceResult<HashMap<String, u32>> {
        self.device_info
            .get(&DynamicMgmtFile::AtrError)
            .map(npu_mgmt::build_atr_error_map)
    }

    /// Returns PCI bus number of the device.
    pub fn busname(&self) -> DeviceResult<String> {
        self.device_info.get(&StaticMgmtFile::Busname)
    }

    /// Returns PCI device ID of the device.
    pub fn pci_dev(&self) -> DeviceResult<String> {
        self.device_info.get(&StaticMgmtFile::Dev)
    }

    /// Returns serial number of the device.
    pub fn device_sn(&self) -> DeviceResult<String> {
        self.device_info.get(&StaticMgmtFile::DeviceSn)
    }

    /// Returns UUID of the device.
    pub fn device_uuid(&self) -> DeviceResult<String> {
        self.device_info.get(&StaticMgmtFile::DeviceUuid)
    }

    /// Retrieves firmware revision from the device.
    pub fn firmware_version(&self) -> DeviceResult<String> {
        self.device_info.get(&DynamicMgmtFile::FwVersion)
    }

    /// Retrieves driver version for the device.
    pub fn driver_version(&self) -> DeviceResult<String> {
        self.device_info.get(&DynamicMgmtFile::Version)
    }

    /// Returns uptime of the device.
    pub fn heartbeat(&self) -> DeviceResult<u32> {
        self.device_info
            .get(&DynamicMgmtFile::Heartbeat)
            .and_then(|str| {
                str.parse::<u32>().map_err(|_| {
                    DeviceError::unexpected_value(format!("Bad heartbeat value: {str}"))
                })
            })
    }

    /// Returns clock frequencies of components in the device.
    pub fn clock_frequency(&self) -> DeviceResult<Vec<ClockFrequency>> {
        self.device_info
            .get(&DynamicMgmtFile::NeClkFreqInfo)
            .map(|str| str.lines().flat_map(ClockFrequency::try_from).collect())
    }

    /// Controls the device led.
    #[allow(dead_code)]
    fn ctrl_device_led(&self, led: (bool, bool, bool)) -> DeviceResult<()> {
        self.device_info.ctrl(
            CtrlFile::DeviceLed,
            &(led.0 as i32 + 0b10 * led.1 as i32 + 0b100 * led.2 as i32).to_string(),
        )
    }

    /// Control NE clocks.
    #[allow(dead_code)]
    fn ctrl_ne_clock(&self, toggle: npu_mgmt::Toggle) -> DeviceResult<()> {
        self.device_info
            .ctrl(CtrlFile::NeClock, &(toggle as u8).to_string())
    }

    /// Control the Dynamic Thermal Management policy.
    #[allow(dead_code)]
    fn ctrl_ne_dtm_policy(&self, policy: npu_mgmt::DtmPolicy) -> DeviceResult<()> {
        self.device_info
            .ctrl(CtrlFile::NeDtmPolicy, &(policy as u8).to_string())
    }

    /// Control NE performance level
    #[allow(dead_code)]
    fn ctrl_performance_level(&self, level: npu_mgmt::PerfLevel) -> DeviceResult<()> {
        self.device_info
            .ctrl(CtrlFile::PerformanceLevel, &(level as u8).to_string())
    }

    /// Control NE performance mode
    #[allow(dead_code)]
    fn ctrl_performance_mode(&self, mode: npu_mgmt::PerfMode) -> DeviceResult<()> {
        self.device_info
            .ctrl(CtrlFile::PerformanceMode, &(mode as u8).to_string())
    }

    /// Retrieve NUMA node ID associated with the NPU's PCI lane
    pub fn numa_node(&self) -> DeviceResult<NumaNode> {
        self.device_info.get_numa_node()
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
            if let Ok(perf_counter) = self.device_info().get_performance_counter(dev_file) {
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
        write!(f, "npu{}", self.device_index())
    }
}

impl Eq for Device {}

impl Ord for Device {
    fn cmp(&self, other: &Self) -> Ordering {
        self.device_index().cmp(&other.device_index())
    }
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.device_info == other.device_info
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

#[derive(Debug)]
pub struct DeviceInfo {
    device_index: u8,
    dev_root: PathBuf,
    sys_root: PathBuf,
    arch: Arch,
    meta: HashMap<&'static str, String>,
    numa_node: Mutex<Option<NumaNode>>,
}

impl DeviceInfo {
    pub(crate) fn new(device_index: u8, dev_root: PathBuf, sys_root: PathBuf) -> DeviceInfo {
        let mut meta = HashMap::default();
        for file in StaticMgmtFile::iter() {
            let filename = file.filename();
            let value = npu_mgmt::read_mgmt_file(&sys_root, filename, device_index).unwrap();
            meta.insert(filename, value);
        }
        let device_type = meta.get(&StaticMgmtFile::DeviceType.filename()).unwrap();
        let soc_rev = meta.get(&StaticMgmtFile::SocRev.filename()).unwrap();
        let arch = Arch::from_str(format!("{device_type}{soc_rev}").as_str())
            .map_err(|_| DeviceError::UnknownArch {
                arch: device_type.clone(),
                rev: soc_rev.clone(),
            })
            .unwrap();
        Self {
            device_index,
            dev_root,
            sys_root,
            arch,
            meta,
            numa_node: Mutex::new(None),
        }
    }

    pub fn arch(&self) -> Arch {
        self.arch
    }

    pub fn get(&self, mgmt_file: &dyn MgmtFile) -> DeviceResult<String> {
        if mgmt_file.is_static() {
            Ok(self.meta.get(mgmt_file.filename()).unwrap().to_string())
        } else {
            let value =
                npu_mgmt::read_mgmt_file(&self.sys_root, mgmt_file.filename(), self.device_index)?;
            Ok(value)
        }
    }

    pub fn ctrl(&self, ctrl_file: CtrlFile, contents: &str) -> DeviceResult<()> {
        npu_mgmt::write_ctrl_file(
            &self.sys_root,
            &ctrl_file.to_string(),
            self.device_index,
            contents,
        )?;

        Ok(())
    }

    pub fn get_numa_node(&self) -> DeviceResult<NumaNode> {
        let mut numa_node = self.numa_node.lock().unwrap();
        if let Some(node) = *numa_node {
            return Ok(node);
        }

        let busname = self.get(&StaticMgmtFile::Busname)?;
        let id = pci::numa::read_numa_node(&self.sys_root, &busname)?
            .parse::<i32>()
            .unwrap();

        let node = if id >= 0 {
            NumaNode::Id(id as usize)
        } else if id == -1 {
            NumaNode::UnSupported
        } else {
            return Err(DeviceError::unexpected_value(format!(
                "Unexpected numa node id: {id}"
            )));
        };

        *numa_node = Some(node);
        Ok(node)
    }

    pub fn get_performance_counter(&self, file: &DeviceFile) -> DeviceResult<PerformanceCounter> {
        PerformanceCounter::read(&self.sys_root, file.filename())
            .map_err(DeviceError::performance_counter_error)
    }
}

impl Eq for DeviceInfo {}

impl PartialEq for DeviceInfo {
    fn eq(&self, other: &Self) -> bool {
        self.device_index == other.device_index
            && self.dev_root == other.dev_root
            && self.sys_root == other.sys_root
            && self.arch == other.arch
            && self.meta == other.meta
            && *self.numa_node.lock().unwrap() == *other.numa_node.lock().unwrap()
    }
}

impl Clone for DeviceInfo {
    fn clone(&self) -> Self {
        Self {
            device_index: self.device_index,
            dev_root: self.dev_root.clone(),
            sys_root: self.sys_root.clone(),
            arch: self.arch,
            meta: self.meta.clone(),
            numa_node: Mutex::new(*self.numa_node.lock().unwrap()),
        }
    }
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
    Range((u8, u8)),
}

impl CoreRange {
    pub fn contains(&self, idx: &CoreIdx) -> bool {
        match self {
            CoreRange::All => true,
            CoreRange::Range((s, e)) => (*s..=*e).contains(idx),
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
    pub(crate) device_index: u8,
    pub(crate) core_range: CoreRange,
    pub(crate) path: PathBuf,
    pub(crate) mode: DeviceMode,
}

impl Display for DeviceFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.file_name().unwrap().to_str().unwrap())
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
    pub fn device_index(&self) -> u8 {
        self.device_index
    }

    /// Returns the range of cores this device file may occupy.
    pub fn core_range(&self) -> CoreRange {
        self.core_range
    }

    /// Return the mode of this device file.
    pub fn mode(&self) -> DeviceMode {
        self.mode
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

        let (device_index, core_indices) = devfs::parse_indices(file_name)?;

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
            device_index,
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
                device_index: 0,
                path: PathBuf::from("./npu0"),
                core_range: CoreRange::All,
                mode: DeviceMode::MultiCore,
            }
        );
        assert!(DeviceFile::try_from(&PathBuf::from("./npu0pe")).is_err());
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0pe0"))?,
            DeviceFile {
                device_index: 0,
                path: PathBuf::from("./npu0pe0"),
                core_range: CoreRange::Range((0, 0)),
                mode: DeviceMode::Single,
            }
        );
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0pe1"))?,
            DeviceFile {
                device_index: 0,
                path: PathBuf::from("./npu0pe1"),
                core_range: CoreRange::Range((1, 1)),
                mode: DeviceMode::Single,
            }
        );
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0pe0-1"))?,
            DeviceFile {
                device_index: 0,
                path: PathBuf::from("./npu0pe0-1"),
                core_range: CoreRange::Range((0, 1)),
                mode: DeviceMode::Fusion,
            }
        );
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0pe0-2"))?,
            DeviceFile {
                device_index: 0,
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
    fn test_static_read_sysfs() -> DeviceResult<()> {
        let device_info = DeviceInfo::new(
            0,
            PathBuf::from("../test_data/test-0/dev"),
            PathBuf::from("../test_data/test-0/sys"),
        );

        assert_eq!(
            device_info.meta.get(StaticMgmtFile::Busname.filename()),
            Some(&String::from("0000:6d:00.0"))
        );
        assert_eq!(
            device_info.get(&StaticMgmtFile::Busname).ok(),
            Some(String::from("0000:6d:00.0"))
        );
        assert_eq!(
            device_info.meta.get(StaticMgmtFile::Busname.filename()),
            Some(&String::from("0000:6d:00.0"))
        );

        Ok(())
    }

    #[test]
    fn test_dynamic_read_sysfs() -> DeviceResult<()> {
        let device_info = DeviceInfo::new(
            0,
            PathBuf::from("../test_data/test-0/dev"),
            PathBuf::from("../test_data/test-0/sys"),
        );

        assert_eq!(
            device_info.meta.get(DynamicMgmtFile::FwVersion.filename()),
            None
        );
        assert_eq!(
            device_info.get(&DynamicMgmtFile::FwVersion).ok(),
            Some(String::from("1.6.0, c1bebfd"))
        );
        assert_eq!(
            device_info.meta.get(DynamicMgmtFile::FwVersion.filename()),
            None
        );

        Ok(())
    }

    #[test]
    fn test_numa_node() -> DeviceResult<()> {
        // npu0 => numa node 0
        let device_info = DeviceInfo::new(
            0,
            PathBuf::from("../test_data/test-0/dev"),
            PathBuf::from("../test_data/test-0/sys"),
        );

        assert_eq!(*device_info.numa_node.lock().unwrap(), None);
        assert_eq!(device_info.get_numa_node()?, NumaNode::Id(0));
        assert_eq!(
            *device_info.numa_node.lock().unwrap(),
            Some(NumaNode::Id(0))
        );

        // npu1 => numa node unsupported
        let device_info = DeviceInfo::new(
            1,
            PathBuf::from("../test_data/test-0/dev"),
            PathBuf::from("../test_data/test-0/sys"),
        );

        assert_eq!(*device_info.numa_node.lock().unwrap(), None);
        assert_eq!(device_info.get_numa_node()?, NumaNode::UnSupported);
        assert_eq!(
            *device_info.numa_node.lock().unwrap(),
            Some(NumaNode::UnSupported)
        );

        Ok(())
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
