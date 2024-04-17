use super::inner::{Config, Cores, Count, DeviceConfigInner};
use crate::arch::Arch;
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

impl From<NotDetermined> for Cores {
    fn from(_: NotDetermined) -> Self {
        Cores(1)
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
pub struct DeviceConfigBuilder<A, N, C> {
    pub(crate) arch: A,
    pub(crate) cores: N,
    pub(crate) count: C,
}

impl<A, C> DeviceConfigBuilder<A, NotDetermined, C> {
    pub fn cores(self, n: u8) -> DeviceConfigBuilder<A, Cores, C> {
        DeviceConfigBuilder {
            arch: self.arch,
            cores: Cores(n),
            count: self.count,
        }
    }
}

impl<A, N, C> DeviceConfigBuilder<A, N, C>
where
    Arch: From<A>,
    Cores: From<N>,
    Count: From<C>,
{
    pub fn count(self, count: u8) -> DeviceConfig {
        let builder = DeviceConfigBuilder {
            arch: self.arch,
            cores: self.cores,
            count: Count::Finite(count),
        };
        builder.build()
    }

    pub fn all(self) -> DeviceConfig {
        let builder = DeviceConfigBuilder {
            arch: self.arch,
            cores: self.cores,
            count: All { _priv: () },
        };

        builder.build()
    }

    pub fn build(self) -> DeviceConfig {
        DeviceConfig {
            inner: DeviceConfigInner {
                cfgs: vec![Config::Unnamed {
                    arch: Arch::from(self.arch),
                    core_num: Cores::from(self.cores).0,
                    count: Count::from(self.count),
                }],
            },
        }
    }
}
