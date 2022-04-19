use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::PathBuf;

use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::fmt::{self, Display, Formatter};

use crate::arch::Arch;
use crate::status::{get_device_status, DeviceStatus};
use crate::{DeviceError, DeviceResult};

#[derive(Debug, Eq, PartialEq)]
pub struct Device {
    device_index: u8,
    arch: Arch,
    cores: Vec<CoreIdx>,
    dev_files: Vec<DeviceFile>,
}

impl Device {
    pub(crate) fn new(
        device_index: u8,
        arch: Arch,
        cores: Vec<CoreIdx>,
        dev_files: Vec<DeviceFile>,
    ) -> Self {
        Self {
            device_index,
            arch,
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

    pub fn arch(&self) -> Arch {
        self.arch
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
            if (file.is_raw() || file.indices().contains(&core))
                && get_device_status(&file.path).await? == DeviceStatus::Occupied
            {
                return Ok(CoreStatus::Occupied(file.to_string()));
            }
        }
        Ok(CoreStatus::Available)
    }

    pub async fn get_status_all(&self) -> DeviceResult<HashMap<CoreIdx, CoreStatus>> {
        let mut status_map: HashMap<CoreIdx, CoreStatus> = self
            .cores
            .iter()
            .map(|k| (*k, CoreStatus::Available))
            .collect();
        for file in &self.dev_files {
            if get_device_status(&file.path).await? == DeviceStatus::Occupied {
                for core in file.indices.iter().chain(
                    file.is_raw()
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CoreStatus {
    Available,
    Occupied(String),
    Unavailable,
}

impl Display for CoreStatus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            CoreStatus::Available => write!(f, "available"),
            CoreStatus::Occupied(devfile) => write!(f, "occupied by {}", devfile),
            CoreStatus::Unavailable => write!(f, "unavailable"),
        }
    }
}

type CoreIdx = u8;

#[derive(Debug, Eq, PartialEq)]
pub struct DeviceFile {
    path: PathBuf,
    indices: Vec<CoreIdx>,
}

impl Display for DeviceFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.file_name().unwrap().to_str().unwrap())
    }
}

impl Ord for DeviceFile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.indices
            .len()
            .cmp(&other.indices.len())
            .then(self.path.cmp(&other.path))
    }
}

impl PartialOrd for DeviceFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl DeviceFile {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn indices(&self) -> &Vec<CoreIdx> {
        &self.indices
    }

    fn is_raw(&self) -> bool {
        self.indices.is_empty()
    }
}

lazy_static! {
    static ref REGEX_RAW: Regex = Regex::new(r"^(npu)(?P<npu>\d*)$").unwrap();
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
        if REGEX_RAW.captures(&item).is_some() {
            Ok(DeviceFile {
                path: path.clone(),
                indices: vec![],
            })
        } else if let Some(x) = REGEX_PE.captures(&item) {
            Ok(DeviceFile {
                path: path.clone(),
                indices: vec![capture_to_str(&x, "pe").parse().unwrap()],
            })
        } else if let Some(x) = REGEX_FUSION.captures(&item) {
            Ok(DeviceFile {
                path: path.clone(),
                indices: capture_to_str(&x, "pe")
                    .split('-')
                    .map(|s| s.parse().unwrap())
                    .collect(),
            })
        } else {
            Err(DeviceError::UnrecognizedDeviceFile(item.to_string()))
        }
    }
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
                indices: vec![]
            }
        );
        assert!(DeviceFile::try_from(&PathBuf::from("./npu0pe")).is_err());
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0pe0"))?,
            DeviceFile {
                path: PathBuf::from("./npu0pe0"),
                indices: vec![0]
            }
        );
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0pe1"))?,
            DeviceFile {
                path: PathBuf::from("./npu0pe1"),
                indices: vec![1]
            }
        );
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0pe0-1"))?,
            DeviceFile {
                path: PathBuf::from("./npu0pe0-1"),
                indices: vec![0, 1]
            }
        );
        assert_eq!(
            DeviceFile::try_from(&PathBuf::from("./npu0pe0-1-2"))?,
            DeviceFile {
                path: PathBuf::from("./npu0pe0-1-2"),
                indices: vec![0, 1, 2]
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