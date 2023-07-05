use super::inner::{Config, Count, DeviceConfigInner};
use crate::arch::Arch;
use crate::device::DeviceMode;
use crate::{DeviceConfig, DeviceError};

#[derive(Clone)]
pub struct NotDetermined {
    pub(crate) _priv: (),
}

impl TryInto<DeviceConfig> for NotDetermined {
    type Error = DeviceError;

    fn try_into(self) -> Result<DeviceConfig, Self::Error> {
        Err(DeviceError::parse_error(
            "",
            "fallback device config is not set",
        ))
    }
}

impl From<NotDetermined> for Arch {
    fn from(_: NotDetermined) -> Self {
        Arch::WarboyB0
    }
}

impl From<NotDetermined> for DeviceMode {
    fn from(_: NotDetermined) -> Self {
        DeviceMode::Fusion
    }
}

impl From<NotDetermined> for Count {
    fn from(_: NotDetermined) -> Self {
        Count::Finite(1)
    }
}

#[derive(Clone)]
pub struct All {
    pub(crate) _priv: (),
}

impl From<All> for Count {
    fn from(_: All) -> Self {
        Count::All
    }
}

/// A builder struct for `DeviceConfig`.
#[derive(Clone)]
pub struct DeviceConfigBuilder<A, M, C> {
    pub(crate) arch: A,
    pub(crate) mode: M,
    pub(crate) count: C,
}

impl<A, C> DeviceConfigBuilder<A, NotDetermined, C> {
    pub fn multicore(self) -> DeviceConfigBuilder<A, DeviceMode, C> {
        DeviceConfigBuilder {
            arch: self.arch,
            mode: DeviceMode::MultiCore,
            count: self.count,
        }
    }

    pub fn single(self) -> DeviceConfigBuilder<A, DeviceMode, C> {
        DeviceConfigBuilder {
            arch: self.arch,
            mode: DeviceMode::Single,
            count: self.count,
        }
    }

    pub fn fused(self) -> DeviceConfigBuilder<A, DeviceMode, C> {
        DeviceConfigBuilder {
            arch: self.arch,
            mode: DeviceMode::Fusion,
            count: self.count,
        }
    }
}

impl<A, M, C> DeviceConfigBuilder<A, M, C>
where
    Arch: From<A>,
    DeviceMode: From<M>,
    Count: From<C>,
{
    pub fn count(self, count: u8) -> DeviceConfig {
        let builder = DeviceConfigBuilder {
            arch: self.arch,
            mode: self.mode,
            count: Count::Finite(count),
        };
        builder.build()
    }

    pub fn all(self) -> DeviceConfig {
        let builder = DeviceConfigBuilder {
            arch: self.arch,
            mode: self.mode,
            count: All { _priv: () },
        };

        builder.build()
    }

    pub fn build(self) -> DeviceConfig {
        let mode = DeviceMode::from(self.mode);
        let core_num = match mode {
            DeviceMode::MultiCore => 0,
            DeviceMode::Single => 1,
            DeviceMode::Fusion => 2,
        };

        DeviceConfig {
            inner: DeviceConfigInner {
                cfgs: vec![Config::Unnamed {
                    arch: Arch::from(self.arch),
                    core_num,
                    mode,
                    count: Count::from(self.count),
                }],
            },
        }
    }
}
