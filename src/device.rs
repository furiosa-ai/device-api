use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

use itertools::Itertools;
use lazy_static::lazy_static;
use regex::{Captures, Regex};

use crate::arch::Arch;
use crate::DeviceError;

#[derive(Debug, Eq, PartialEq)]
pub struct Device {
    device_index: u8,
    path: PathBuf,
    mode: DeviceMode,
    arch: Arch,
    status: DeviceStatus,
}

impl Device {
    pub(crate) fn new(
        device_index: u8,
        path: PathBuf,
        mode: DeviceMode,
        arch: Arch,
        status: DeviceStatus,
    ) -> Self {
        Self {
            device_index,
            path,
            mode,
            arch,
            status,
        }
    }

    pub(crate) fn change_status(self, status: DeviceStatus) -> Self {
        Self { status, ..self }
    }

    pub fn name(&self) -> String {
        format!("npu{}:{}", self.device_index, self.mode)
    }

    #[deprecated]
    pub fn devname(&self) -> String {
        format!("npu{}pe{}", self.device_index, self.mode)
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn mode(&self) -> &DeviceMode {
        &self.mode
    }

    pub fn device_index(&self) -> u8 {
        self.device_index
    }

    pub fn arch(&self) -> Arch {
        self.arch
    }

    pub fn status(&self) -> DeviceStatus {
        self.status
    }

    pub fn available(&self) -> bool {
        matches!(self.status, DeviceStatus::Available)
    }

    pub fn core_num(&self) -> u8 {
        self.mode.count()
    }

    pub fn single_core(&self) -> bool {
        self.core_num() == 1
    }

    pub fn fused(&self) -> bool {
        self.core_num() > 1
    }
}

impl Display for Device {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "npu:{}:{} {} {}",
            self.device_index, self.mode, self.arch, self.status
        )
    }
}

impl Ord for Device {
    fn cmp(&self, other: &Self) -> Ordering {
        self.device_index
            .cmp(&other.device_index)
            .then(self.mode.cmp(&other.mode))
    }
}

impl PartialOrd for Device {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, strum_macros::Display)]
#[strum(serialize_all = "kebab_case")]
pub enum DeviceStatus {
    Available,
    Occupied,
    Fused,
    Unavailable,
}

type CoreIdx = u8;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DeviceMode {
    Single(CoreIdx),
    Fusion(Vec<CoreIdx>),
}

impl Display for DeviceMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            DeviceMode::Single(pe) => format!("{}", pe),
            DeviceMode::Fusion(v) => v.iter().join("-"),
        };

        write!(f, "{}", name)
    }
}

impl DeviceMode {
    fn count(&self) -> u8 {
        match self {
            DeviceMode::Single(_) => 1,
            DeviceMode::Fusion(v) => u8::try_from(v.len()).unwrap(),
        }
    }
}

lazy_static! {
    static ref REGEX_PE: Regex = Regex::new(r"^(npu)(?P<npu>\d*)(pe)(?P<pe>\d+)$").unwrap();
    static ref REGEX_FUSION: Regex =
        Regex::new(r"^(npu)(?P<npu>\d*)(pe)(?P<pe>(\d+-)+\d+)$").unwrap();
}

fn capture_to_str<'a>(c: &'a Captures<'_>, key: &'a str) -> &'a str {
    c.name(key).unwrap().as_str()
}

impl TryFrom<&str> for DeviceMode {
    type Error = DeviceError;

    fn try_from(item: &str) -> Result<Self, Self::Error> {
        if let Some(x) = REGEX_PE.captures(item) {
            Ok(DeviceMode::Single(
                capture_to_str(&x, "pe").parse().unwrap(),
            ))
        } else if let Some(x) = REGEX_FUSION.captures(item) {
            let indexes: Vec<u8> = capture_to_str(&x, "pe")
                .split('-')
                .map(|s| s.parse().unwrap())
                .collect();

            Ok(DeviceMode::Fusion(indexes))
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
        assert!(DeviceMode::try_from("npu0").is_err());
        assert!(DeviceMode::try_from("npu0pe").is_err());
        assert_eq!(DeviceMode::try_from("npu0pe0")?, DeviceMode::Single(0));
        assert_eq!(DeviceMode::try_from("npu0pe1")?, DeviceMode::Single(1));
        assert_eq!(
            DeviceMode::try_from("npu0pe0-1")?,
            DeviceMode::Fusion(vec![0, 1])
        );
        assert_eq!(
            DeviceMode::try_from("npu0pe0-1-2")?,
            DeviceMode::Fusion(vec![0, 1, 2])
        );
        assert!(DeviceMode::try_from("npu0pe0-").is_err());
        assert!(DeviceMode::try_from("npu0pe-1").is_err());
        Ok(())
    }

    #[test]
    fn test_fmt() {
        assert_eq!(format!("{}", DeviceMode::Single(0)), "0");
        assert_eq!(format!("{}", DeviceMode::Single(1)), "1");

        assert_eq!(format!("{}", DeviceMode::Fusion(vec![0, 1])), "0-1");
        assert_eq!(
            format!("{}", DeviceMode::Fusion(vec![0, 1, 2, 3])),
            "0-1-2-3"
        );
    }
}
