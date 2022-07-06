use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::PathBuf;

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use crate::arch::Arch;
use crate::hwmon;
use crate::status::{get_device_status, DeviceStatus};
use crate::{devfs, sysfs, DeviceError, DeviceResult};

#[derive(Debug, Eq, PartialEq)]

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
/// [`find_devices`][crate::find_devices]).
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
    pub fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    /// Returns `Arch` of the device(e.g., `Warboy`).
    pub fn arch(&self) -> Arch {
        self.device_info().arch()
    }

    /// Returns a liveness state of the device.
    pub fn alive(&mut self) -> DeviceResult<bool> {
        self.device_info
            .get(sysfs::npu_mgmt::ALIVE)
            .and_then(sysfs::npu_mgmt::parse_zero_or_one_to_bool)
    }

    /// Returns error states of the device.
    pub fn atr_error(&mut self) -> DeviceResult<HashMap<String, u32>> {
        self.device_info
            .get(sysfs::npu_mgmt::ATR_ERROR)
            .map(sysfs::npu_mgmt::build_atr_error_map)
    }

    /// Returns PCI bus number of the device.
    pub fn busname(&mut self) -> DeviceResult<&str> {
        self.device_info
            .get(sysfs::npu_mgmt::BUSNAME)
            .map(String::as_str)
    }

    /// Returns PCI device ID of the device.
    pub fn pci_dev(&mut self) -> DeviceResult<&str> {
        self.device_info
            .get(sysfs::npu_mgmt::DEV)
            .map(String::as_str)
    }

    /// Retrieves firmware revision from the device.
    pub fn firmware_version(&mut self) -> DeviceResult<&str> {
        self.device_info
            .get(sysfs::npu_mgmt::FW_VERSION)
            .map(String::as_str)
    }

    /// Returns uptime of the device.
    pub fn heartbeat(&mut self) -> DeviceResult<u32> {
        self.device_info
            .get(sysfs::npu_mgmt::HEARTBEAT)
            .and_then(|str| {
                str.parse::<u32>().map_err(|_| {
                    DeviceError::unexpected_value(format!("Bad heartbeat value: {}", str))
                })
            })
    }

    /// Controls the device led.
    pub fn ctrl_device_led(&mut self, led: (bool, bool, bool)) -> DeviceResult<()> {
        self.device_info.ctrl(
            sysfs::npu_mgmt::DEVICE_LED,
            &(led.0 as i32 + 0b10 * led.1 as i32 + 0b100 * led.2 as i32).to_string(),
        )
    }

    /// Control NE clocks.
    pub fn ctrl_ne_clock(&mut self, toggle: sysfs::npu_mgmt::Toggle) -> DeviceResult<()> {
        self.device_info
            .ctrl(sysfs::npu_mgmt::NE_CLOCK, &(toggle as u8).to_string())
    }

    /// Control the Dynamic Thermal Management policy.
    pub fn ctrl_ne_dtm_policy(&mut self, policy: sysfs::npu_mgmt::DtmPolicy) -> DeviceResult<()> {
        self.device_info
            .ctrl(sysfs::npu_mgmt::NE_DTM_POLICY, &(policy as u8).to_string())
    }

    /// Control NE performance level
    pub fn ctrl_performance_level(
        &mut self,
        level: sysfs::npu_mgmt::PerfLevel,
    ) -> DeviceResult<()> {
        self.device_info.ctrl(
            sysfs::npu_mgmt::PERFORMANCE_LEVEL,
            &(level as u8).to_string(),
        )
    }

    /// Control NE performance mode
    pub fn ctrl_performance_mode(&mut self, mode: sysfs::npu_mgmt::PerfMode) -> DeviceResult<()> {
        self.device_info
            .ctrl(sysfs::npu_mgmt::PERFORMANCE_MODE, &(mode as u8).to_string())
    }

    /// Retrieve NUMA node ID associated with the NPU's PCI lane
    pub fn numa_node(&mut self) -> DeviceResult<usize> {
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

    /// Examine a specific core of the device, whether it is available or not.
    pub async fn get_status_core(&self, core: CoreIdx) -> DeviceResult<CoreStatus> {
        for file in &self.dev_files {
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

impl Ord for Device {
    fn cmp(&self, other: &Self) -> Ordering {
        self.device_index().cmp(&other.device_index())
    }
}

impl PartialOrd for Device {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct DeviceInfo {
    device_index: u8,
    dev_root: PathBuf,
    sys_root: PathBuf,
    meta: DeviceMetadata,
    numa_node: Option<usize>,
}

impl DeviceInfo {
    pub(crate) fn new(
        device_index: u8,
        dev_root: PathBuf,
        sys_root: PathBuf,
        meta: DeviceMetadata,
    ) -> DeviceInfo {
        Self {
            device_index,
            dev_root,
            sys_root,
            meta,
            numa_node: None,
        }
    }

    pub fn arch(&self) -> Arch {
        self.meta.arch
    }

    pub fn get(&mut self, key: &str) -> DeviceResult<&String> {
        let (key, _) = sysfs::npu_mgmt::MGMT_FILES
            .iter()
            .find(|mgmt_file| mgmt_file.0 == key)
            .ok_or_else(|| DeviceError::unsupported_key(key))?;

        Ok(self
            .meta
            .map
            .entry(key)
            .or_insert(sysfs::npu_mgmt::read_mgmt_file(
                &self.sys_root,
                key,
                self.device_index,
            )?))
    }

    pub fn ctrl(&mut self, key: &str, contents: &str) -> DeviceResult<()> {
        let key = sysfs::npu_mgmt::CTRL_FILES
            .iter()
            .find(|ctrl| **ctrl == key)
            .ok_or_else(|| DeviceError::unsupported_key(key))?;

        sysfs::npu_mgmt::write_ctrl_file(&self.sys_root, key, self.device_index, contents)?;

        if let Some((key, _)) = sysfs::npu_mgmt::MGMT_FILES
            .iter()
            .find(|mgmt_file| mgmt_file.0 == *key)
        {
            self.meta.map.remove(key);
        }

        Ok(())
    }

    pub fn get_numa_node(&mut self) -> DeviceResult<usize> {
        let numa_node = match self.numa_node {
            Some(numa_node) => numa_node,
            None => {
                // note for .clone(): see https://doc.rust-lang.org/nomicon/lifetime-mismatch.html
                let busname = self.get(sysfs::npu_mgmt::BUSNAME)?.clone();

                sysfs::pci::numa::read_numa_node(&self.sys_root, &busname)?
                    .parse::<usize>()
                    .unwrap()
            }
        };

        Ok(numa_node)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct DeviceMetadata {
    pub(crate) arch: Arch,
    pub(crate) map: HashMap<&'static str, String>,
}

impl TryFrom<HashMap<&'static str, String>> for DeviceMetadata {
    type Error = DeviceError;

    fn try_from(map: HashMap<&'static str, String>) -> Result<Self, Self::Error> {
        use sysfs::npu_mgmt::*;

        let device_type = map
            .get(DEVICE_TYPE)
            .ok_or_else(|| DeviceError::file_not_found(DEVICE_TYPE))?;
        let arch = Arch::from_str(device_type).map_err(|_| DeviceError::UnknownArch {
            arch: device_type.clone(),
        })?;

        Ok(Self { arch, map })
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
            CoreStatus::Occupied(devfile) => write!(f, "occupied by {}", devfile),
            CoreStatus::Unavailable => write!(f, "unavailable"),
        }
    }
}

pub(crate) type CoreIdx = u8;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CoreRange {
    All,
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
#[derive(Debug, Eq, PartialEq, Clone)]
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

        let (device_index, core_indices) = devfs::parse_indices(&file_name)?;

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
#[derive(Debug, Eq, PartialEq, Copy, Clone, enum_utils::FromStr)]
#[enumeration(case_insensitive)]
pub enum DeviceMode {
    Single,
    Fusion,
    MultiCore,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysfs::npu_mgmt::read_mgmt_files;

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
    fn test_numa_node() -> DeviceResult<()> {
        // npu0 => numa node 0
        let device_meta = DeviceMetadata::try_from(read_mgmt_files("test_data/test-0/sys", 0)?)?;
        let mut device_info = DeviceInfo::new(
            0,
            PathBuf::from("test_data/test-0/dev"),
            PathBuf::from("test_data/test-0/sys"),
            device_meta,
        );

        assert_eq!(device_info.get_numa_node()?, 0);

        // npu1 => numa node 1
        let device_meta = DeviceMetadata::try_from(read_mgmt_files("test_data/test-0/sys", 1)?)?;
        let mut device_info = DeviceInfo::new(
            0,
            PathBuf::from("test_data/test-0/dev"),
            PathBuf::from("test_data/test-0/sys"),
            device_meta,
        );

        assert_eq!(device_info.get_numa_node()?, 1);

        Ok(())
    }
}
