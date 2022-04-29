use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::PathBuf;

use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::fmt::{self, Display, Formatter};
use std::io;
use std::io::ErrorKind;
use std::str::FromStr;

use crate::arch::Arch;
use crate::status::{get_device_status, DeviceStatus};
use crate::{sysfs, DeviceError, DeviceResult};

#[derive(Debug, Eq, PartialEq)]
pub struct Device {
    device_index: u8,
    device_info: DeviceInfo,
    pub(crate) cores: Vec<CoreIdx>,
    pub(crate) dev_files: Vec<DeviceFile>,
}

impl Device {
    pub(crate) fn new(
        device_index: u8,
        device_info: DeviceInfo,
        cores: Vec<CoreIdx>,
        dev_files: Vec<DeviceFile>,
    ) -> Self {
        Self {
            device_index,
            device_info,
            cores,
            dev_files,
        }
    }

    pub fn name(&self) -> String {
        format!("npu{}", self.device_index)
    }

    pub fn device_index(&self) -> u8 {
        self.device_index
    }

    pub fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    pub fn arch(&self) -> Arch {
        self.device_info().arch()
    }

    pub fn busname(&self) -> Option<&str> {
        self.device_info().busname()
    }

    pub fn pci_dev(&self) -> Option<&str> {
        self.device_info().pci_dev()
    }

    pub fn firmware_version(&self) -> Option<&str> {
        self.device_info().firmware_version()
    }

    pub fn core_num(&self) -> u8 {
        u8::try_from(self.cores.len()).unwrap()
    }

    pub fn cores(&self) -> &Vec<CoreIdx> {
        &self.cores
    }

    pub fn dev_files(&self) -> &Vec<DeviceFile> {
        &self.dev_files
    }

    pub async fn get_status_core(&self, core: CoreIdx) -> DeviceResult<CoreStatus> {
        for file in &self.dev_files {
            if (file.is_multicore() || file.indices().contains(&core))
                && get_device_status(&file.path).await? == DeviceStatus::Occupied
            {
                return Ok(CoreStatus::Occupied(file.to_string()));
            }
        }
        Ok(CoreStatus::Available)
    }

    pub async fn get_status_all(&self) -> DeviceResult<HashMap<CoreIdx, CoreStatus>> {
        let mut status_map = self.new_status_map();

        for file in &self.dev_files {
            if get_device_status(&file.path).await? == DeviceStatus::Occupied {
                for core in file.indices.iter().chain(
                    file.is_multicore()
                        .then(|| self.cores.iter())
                        .into_iter()
                        .flatten(),
                ) {
                    status_map.insert(*core, CoreStatus::Occupied(file.to_string()));
                }
            }
        }
        Ok(status_map)
    }

    pub(crate) fn new_status_map(&self) -> HashMap<CoreIdx, CoreStatus> {
        self.cores
            .iter()
            .map(|k| (*k, CoreStatus::Available))
            .collect()
    }
}

impl Display for Device {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "npu{}", self.device_index)
    }
}

impl Ord for Device {
    fn cmp(&self, other: &Self) -> Ordering {
        self.device_index.cmp(&other.device_index)
    }
}

impl PartialOrd for Device {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct DeviceInfo {
    arch: Arch,
    busname: Option<String>,
    pci_dev: Option<String>,
    firmware_version: Option<String>,
}

impl DeviceInfo {
    pub(crate) fn new(
        arch: Arch,
        busname: Option<String>,
        pci_dev: Option<String>,
        firmware_version: Option<String>,
    ) -> DeviceInfo {
        Self {
            arch,
            busname,
            pci_dev,
            firmware_version,
        }
    }

    pub fn arch(&self) -> Arch {
        self.arch
    }

    pub fn busname(&self) -> Option<&str> {
        self.busname.as_deref()
    }

    pub fn pci_dev(&self) -> Option<&str> {
        self.pci_dev.as_deref()
    }

    pub fn firmware_version(&self) -> Option<&str> {
        self.firmware_version.as_deref()
    }
}

impl TryFrom<HashMap<&'static str, String>> for DeviceInfo {
    type Error = DeviceError;

    fn try_from(mut map: HashMap<&'static str, String>) -> Result<Self, Self::Error> {
        use sysfs::npu_mgmt::*;

        let contents = map.remove(DEVICE_TYPE).ok_or_else(|| {
            io::Error::new(ErrorKind::NotFound, format!("{} not found", DEVICE_TYPE))
        })?;
        let arch = Arch::from_str(&contents).map_err(|_| DeviceError::UnknownArch {
            arch: contents.to_string(),
        })?;

        let busname = map.remove(BUSNAME);
        let pci_dev = map.remove(DEV);
        let firmware_version = map.remove(FW_VERSION);

        Ok(DeviceInfo::new(arch, busname, pci_dev, firmware_version))
    }
}

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

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DeviceFile {
    pub(crate) path: PathBuf,
    pub(crate) indices: Vec<CoreIdx>,
    pub(crate) mode: DeviceMode,
}

impl Display for DeviceFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.file_name().unwrap().to_str().unwrap())
    }
}

impl DeviceFile {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn filename(&self) -> &str {
        // We should guarantee that it returns a filename
        self.path
            .file_name()
            .expect("not a file")
            .to_str()
            .expect("invalid UTF-8 encoding")
    }

    pub fn indices(&self) -> &Vec<CoreIdx> {
        &self.indices
    }

    pub fn mode(&self) -> DeviceMode {
        self.mode
    }

    pub(crate) fn is_multicore(&self) -> bool {
        self.mode == DeviceMode::MultiCore
    }
}

lazy_static! {
    static ref REGEX_MULTICORE: Regex = Regex::new(r"^(npu)(?P<npu>\d*)$").unwrap();
    static ref REGEX_PE: Regex = Regex::new(r"^(npu)(?P<npu>\d*)(pe)(?P<pe>\d+)$").unwrap();
    static ref REGEX_FUSION: Regex =
        Regex::new(r"^(npu)(?P<npu>\d*)(pe)(?P<pe>(\d+-)+\d+)$").unwrap();
}

fn capture_to_str<'a>(c: &'a Captures<'_>, key: &'a str) -> &'a str {
    c.name(key).unwrap().as_str()
}

impl TryFrom<&PathBuf> for DeviceFile {
    type Error = DeviceError;

    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let item = path
            .file_name()
            .expect("not a file")
            .to_string_lossy()
            .to_string();
        if REGEX_MULTICORE.captures(&item).is_some() {
            Ok(DeviceFile {
                path: path.clone(),
                indices: vec![],
                mode: DeviceMode::MultiCore,
            })
        } else if let Some(x) = REGEX_PE.captures(&item) {
            Ok(DeviceFile {
                path: path.clone(),
                indices: vec![capture_to_str(&x, "pe").parse().unwrap()],
                mode: DeviceMode::Single,
            })
        } else if let Some(x) = REGEX_FUSION.captures(&item) {
            Ok(DeviceFile {
                path: path.clone(),
                indices: capture_to_str(&x, "pe")
                    .split('-')
                    .map(|s| s.parse().unwrap())
                    .collect(),
                mode: DeviceMode::Fusion,
            })
        } else {
            Err(DeviceError::IncompatibleDriver {
                cause: format!("{} file cannot be recognized", path.display()),
            })
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum DeviceMode {
    Single,
    Fusion,
    MultiCore,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from() -> Result<(), DeviceError> {
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0"))?,
            DeviceFile {
                path: PathBuf::from("./npu0"),
                indices: vec![],
                mode: DeviceMode::MultiCore,
            }
        );
        assert!(DeviceFile::try_from(&PathBuf::from("./npu0pe")).is_err());
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0pe0"))?,
            DeviceFile {
                path: PathBuf::from("./npu0pe0"),
                indices: vec![0],
                mode: DeviceMode::Single,
            }
        );
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0pe1"))?,
            DeviceFile {
                path: PathBuf::from("./npu0pe1"),
                indices: vec![1],
                mode: DeviceMode::Single,
            }
        );
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0pe0-1"))?,
            DeviceFile {
                path: PathBuf::from("./npu0pe0-1"),
                indices: vec![0, 1],
                mode: DeviceMode::Fusion,
            }
        );
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0pe0-1-2"))?,
            DeviceFile {
                path: PathBuf::from("./npu0pe0-1-2"),
                indices: vec![0, 1, 2],
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
}
