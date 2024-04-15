use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use strum_macros::{AsRefStr, EnumIter};

use crate::device::DeviceInner;
use crate::sysfs::npu_mgmt;
use crate::{arch_impl, DeviceResult};

/// Enum for the NPU architecture.
#[derive(
    AsRefStr, Clone, Copy, Debug, enum_utils::FromStr, Eq, PartialEq, PartialOrd, EnumIter,
)]
#[enumeration(case_insensitive)]
// the order here is important, it will be used to determine the index of the devices
pub enum Arch {
    #[enumeration(alias = "Warboy")]
    WarboyB0,
    Renegade,
}

impl Arch {
    pub(crate) fn devfile_path<P: AsRef<Path>>(&self, devfs: P) -> PathBuf {
        match self {
            Arch::WarboyB0 => devfs.as_ref().to_path_buf(),
            Arch::Renegade => devfs.as_ref().join("renegade"),
        }
    }

    pub(crate) fn create_inner(
        &self,
        idx: u8,
        _devfs: &str,
        sysfs: &str,
    ) -> DeviceResult<Box<dyn DeviceInner>> {
        match self {
            Arch::WarboyB0 => arch_impl::WarboyInner::new(self.clone(), idx, sysfs.into())
                .map(|t| Box::new(t) as Box<dyn DeviceInner>),
            Arch::Renegade => arch_impl::RenegadeInner::new(self.clone(), idx, sysfs.into())
                .map(|t| Box::new(t) as Box<dyn DeviceInner>),
        }
    }

    pub(crate) fn platform_type_path(&self, idx: u8, sysfs: &str) -> PathBuf {
        let platform_type = npu_mgmt::file::PLATFORM_TYPE;
        match self {
            Arch::WarboyB0 => {
                PathBuf::from(sysfs).join(format!("class/npu_mgmt/npu{idx}_mgmt/{platform_type}"))
            }
            Arch::Renegade => PathBuf::from(sysfs).join(format!(
                "class/renegade_mgmt/renegade!npu{idx}mgmt/{platform_type}"
            )),
        }
    }
}

impl Display for Arch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Arch::*;

        // Keep the same as npu-id of Compiler to display
        match self {
            WarboyB0 => write!(f, "warboy"),
            Renegade => write!(f, "renegade"),
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
    fn test_arch_devfile_dir() {
        let warboy = Arch::WarboyB0;
        let renegade = Arch::Renegade;

        assert_eq!(warboy.devfile_path("/dev"), PathBuf::from("/dev"));
        assert_eq!(
            renegade.devfile_path("/dev"),
            PathBuf::from("/dev/renegade")
        );
    }

    #[test]
    fn test_arch_path_platform_type() {
        let warboy = Arch::WarboyB0;
        let renegade = Arch::Renegade;

        assert_eq!(
            warboy.platform_type_path(3, "/sys"),
            PathBuf::from("/sys/class/npu_mgmt/npu3_mgmt/platform_type")
        );

        assert_eq!(
            renegade.platform_type_path(3, "/sys"),
            PathBuf::from("/sys/class/renegade_mgmt/renegade!npu3mgmt/platform_type")
        );
    }

    #[test]
    fn test_arch_order() {
        use strum::IntoEnumIterator;
        let mut iter = Arch::iter();
        assert_eq!(iter.next(), Some(Arch::WarboyB0));
        assert_eq!(iter.next(), Some(Arch::Renegade));
        assert_eq!(iter.next(), None);
    }
}
