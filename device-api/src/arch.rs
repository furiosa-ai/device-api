use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use strum_macros::AsRefStr;

use crate::arch_impl;
use crate::device::DeviceInner;
use crate::sysfs::npu_mgmt;

/// Enum for the NPU architecture.
#[derive(AsRefStr, Clone, Copy, Debug, enum_utils::FromStr, Eq, PartialEq, PartialOrd)]
#[enumeration(case_insensitive)]
pub enum Arch {
    WarboyA0,
    #[enumeration(alias = "Warboy")]
    WarboyB0,
    Renegade,
    U250, /* TODO - It's somewhat ambiguous. We need two attributes to distinguish both HW type
           * and NPU family. */
}

#[derive(Clone, Copy, Debug)]
pub enum ArchFamily {
    Warboy,
    Renegade,
}

impl ArchFamily {
    pub(crate) fn devfile_dir<P: AsRef<Path>>(self, devfs: P) -> PathBuf {
        match self {
            ArchFamily::Warboy => devfs.as_ref().to_path_buf(),
            ArchFamily::Renegade => devfs.as_ref().join("renegade"),
        }
    }

    pub(crate) fn create_inner(self, idx: u8, _devfs: &str, sysfs: &str) -> Box<dyn DeviceInner> {
        match self {
            ArchFamily::Warboy => Box::new(arch_impl::WarboyInner::new(idx, sysfs.into())),
            ArchFamily::Renegade => Box::new(arch_impl::RenegadeInner::new(idx, sysfs.into())),
        }
    }

    pub(crate) fn path_platform_type(self, idx: u8, sysfs: &str) -> PathBuf {
        let platform_type = npu_mgmt::file::PLATFORM_TYPE;
        match self {
            ArchFamily::Warboy => {
                PathBuf::from(sysfs).join(format!("class/npu_mgmt/npu{idx}_mgmt/{platform_type}"))
            }
            ArchFamily::Renegade => PathBuf::from(sysfs).join(format!(
                "class/renegade_mgmt/renegade!npu{idx}mgmt/{platform_type}"
            )),
        }
    }
}

impl From<Arch> for ArchFamily {
    fn from(arch: Arch) -> Self {
        match arch {
            Arch::WarboyA0 | Arch::WarboyB0 => ArchFamily::Warboy,
            Arch::Renegade => ArchFamily::Renegade,
            Arch::U250 => ArchFamily::Warboy,
        }
    }
}

impl Display for Arch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Arch::*;

        // Keep the same as npu-id of Compiler to display
        match self {
            WarboyA0 => write!(f, "warboy-a0"),
            WarboyB0 => write!(f, "warboy"),
            Renegade => write!(f, "renegade"),
            U250 => write!(f, "u250"),
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
        let warboy = ArchFamily::Warboy;
        let renegade = ArchFamily::Renegade;

        assert_eq!(warboy.devfile_dir("/dev"), PathBuf::from("/dev"));
        assert_eq!(renegade.devfile_dir("/dev"), PathBuf::from("/dev/renegade"));
    }

    #[test]
    fn test_family_path_platform_type() {
        let warboy = ArchFamily::Warboy;
        let renegade = ArchFamily::Renegade;

        assert_eq!(
            warboy.path_platform_type(3, "/sys"),
            PathBuf::from("/sys/class/npu_mgmt/npu3_mgmt/platform_type")
        );

        assert_eq!(
            renegade.path_platform_type(3, "/sys"),
            PathBuf::from("/sys/class/renegade_mgmt/renegade!npu3mgmt/platform_type")
        );
    }
}
