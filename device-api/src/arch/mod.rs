mod rngd;
mod warboy;

use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use rngd::RNGDInner;
use strum_macros::{AsRefStr, EnumIter};
use warboy::WarboyInner;

use crate::device::DeviceInner;
use crate::sysfs::npu_mgmt;
use crate::DeviceResult;

/// Enum for the NPU architecture.
#[derive(
    AsRefStr, Clone, Copy, Debug, enum_utils::FromStr, Eq, PartialEq, PartialOrd, EnumIter,
)]
#[enumeration(case_insensitive)]
pub enum Arch {
    #[enumeration(alias = "Warboy")]
    WarboyB0,
    RNGD,
}

impl Arch {
    pub fn num_pe(&self) -> u8 {
        match self {
            Arch::WarboyB0 => 2,
            Arch::Renegade => 8,
        }
    }

    pub fn is_fusible_count(&self, count: u8) -> bool {
        match self {
            Arch::WarboyB0 => matches!(count, 1 | 2),
            Arch::Renegade => matches!(count, 1 | 2 | 4),
        }
    }

    pub(crate) fn devfile_path<P: AsRef<Path>>(&self, devfs: P) -> PathBuf {
        match self {
            Arch::WarboyB0 => devfs.as_ref().to_path_buf(),
            Arch::RNGD => devfs.as_ref().join("rngd"),
        }
    }

    pub(crate) fn create_inner(
        &self,
        idx: u8,
        _devfs: &str,
        sysfs: &str,
    ) -> DeviceResult<Box<dyn DeviceInner>> {
        match self {
            Arch::WarboyB0 => {
                WarboyInner::new(idx, sysfs.into()).map(|t| Box::new(t) as Box<dyn DeviceInner>)
            }
            Arch::RNGD => {
                RNGDInner::new(idx, sysfs.into()).map(|t| Box::new(t) as Box<dyn DeviceInner>)
            }
        }
    }

    pub(crate) fn platform_type_path(&self, idx: u8, sysfs: &str) -> PathBuf {
        let platform_type = npu_mgmt::file::PLATFORM_TYPE;
        match self {
            Arch::WarboyB0 => {
                PathBuf::from(sysfs).join(format!("class/npu_mgmt/npu{idx}_mgmt/{platform_type}"))
            }
            Arch::RNGD => PathBuf::from(sysfs)
                .join(format!("class/rngd_mgmt/rngd!npu{idx}mgmt/{platform_type}")),
        }
    }
}

impl Display for Arch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Arch::*;

        // Keep the same as npu-id of Compiler to display
        match self {
            WarboyB0 => write!(f, "warboy"),
            RNGD => write!(f, "rngd"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_archkind() {
        assert!(Arch::from_str("Warboy").is_ok());
    }

    #[test]
    fn test_family_devfile_dir() {
        let warboy = Arch::WarboyB0;
        let rngd = Arch::RNGD;

        assert_eq!(warboy.devfile_path("/dev"), PathBuf::from("/dev"));
        assert_eq!(rngd.devfile_path("/dev"), PathBuf::from("/dev/rngd"));
    }

    #[test]
    fn test_family_path_platform_type() {
        let warboy = Arch::WarboyB0;
        let rngd = Arch::RNGD;

        assert_eq!(
            warboy.platform_type_path(3, "/sys"),
            PathBuf::from("/sys/class/npu_mgmt/npu3_mgmt/platform_type")
        );

        assert_eq!(
            rngd.platform_type_path(3, "/sys"),
            PathBuf::from("/sys/class/rngd_mgmt/rngd!npu3mgmt/platform_type")
        );
    }
}
