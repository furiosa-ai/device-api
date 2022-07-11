use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::{all_consuming, map, map_res, opt};
use nom::sequence::{delimited, preceded, separated_pair};
use nom::Parser;

use crate::arch::Arch;
use crate::device::{CoreIdx, CoreRange, CoreStatus, Device, DeviceFile, DeviceMode};
use crate::error::DeviceResult;

/// Describes a required set of devices for [`find_devices`][crate::find_devices].
///
/// # Examples
/// ```rust
/// use furiosa_device::DeviceConfig;
///
/// // 1 core
/// DeviceConfig::warboy().build();
///
/// // 1 core x 2
/// DeviceConfig::warboy().count(2);
///
/// // Fused 2 cores x 2
/// DeviceConfig::warboy().fused().count(2);
/// ```
///
/// See also [struct `Device`][`Device`].
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DeviceConfig {
    // TODO: Named cannot describe MultiCore yet.
    Named {
        device_id: u8,
        core_range: CoreRange,
    },
    Unnamed {
        arch: Arch,
        core_num: u8,
        mode: DeviceMode,
        count: u8,
    },
}

impl DeviceConfig {
    /// Returns a builder associated with Warboy NPUs.
    pub fn warboy() -> DeviceConfigBuilder<Arch, NotDetermined, NotDetermined> {
        DeviceConfigBuilder {
            arch: Arch::Warboy,
            mode: NotDetermined,
            count: NotDetermined,
        }
    }

    pub(crate) fn fit(&self, arch: Arch, device_file: &DeviceFile) -> bool {
        match self {
            Self::Named {
                device_id,
                core_range,
            } => {
                device_file.device_index() == *device_id && device_file.core_range() == *core_range
            }
            Self::Unnamed {
                arch: config_arch,
                core_num: _,
                mode,
                count: _,
            } => arch == *config_arch && device_file.mode() == *mode,
        }
    }

    pub(crate) fn count(&self) -> u8 {
        match self {
            Self::Named {
                device_id: _,
                core_range: _,
            } => 1,
            Self::Unnamed {
                arch: _,
                core_num: _,
                mode: _,
                count,
            } => *count,
        }
    }
}

impl Default for DeviceConfig {
    fn default() -> Self {
        DeviceConfig::warboy().fused().count(1)
    }
}

impl FromStr for DeviceConfig {
    type Err = nom::Err<()>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn digit_to_u8<'a>() -> impl FnMut(&'a str) -> nom::IResult<&'a str, u8, ()> {
            map_res(digit1, |s: &str| s.parse::<u8>())
        }
        // try parsing named configs, from patterns e.g., "0:0" or "0:0-1"
        let parsed_named = all_consuming::<_, _, (), _>(digit_to_u8().and(opt(preceded(
            tag(":"),
            alt((
                map_res(
                    separated_pair(digit_to_u8(), tag("-"), digit_to_u8()),
                    CoreRange::try_from,
                ),
                map(digit_to_u8(), CoreRange::from),
            )),
        ))))(s);

        match parsed_named {
            Ok((_, (device_id, core_id))) => {
                let core_range = core_id.unwrap_or(CoreRange::All);
                Ok(DeviceConfig::Named {
                    device_id,
                    core_range,
                })
            }
            Err(_) => {
                // try parsing unnamed configs, from patterns e.g., "warboy*1" or "warboy(1)*2"
                let (_, ((arch, mode), count)) = all_consuming(separated_pair(
                    map_res(tag("warboy"), |s: &str| s.parse::<Arch>()).and(opt(delimited(
                        tag("("),
                        digit_to_u8(),
                        tag(")"),
                    ))),
                    tag("*"),
                    digit_to_u8(),
                ))(s)?;
                let (core_num, mode) = match mode {
                    None => (0, DeviceMode::MultiCore),
                    Some(1) => (1, DeviceMode::Single),
                    // TODO: Improve below
                    Some(n) => (n, DeviceMode::Fusion),
                };

                Ok(DeviceConfig::Unnamed {
                    arch,
                    core_num,
                    mode,
                    count,
                })
            }
        }
    }
}

impl Display for DeviceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Named {
                device_id,
                core_range,
            } => match core_range {
                CoreRange::All => {
                    write!(f, "{}", device_id)
                }
                CoreRange::Range((s, e)) => {
                    if s == e {
                        write!(f, "{}:{}", device_id, s)
                    } else {
                        write!(f, "{}:{}-{}", device_id, s, e)
                    }
                }
            },
            Self::Unnamed {
                arch,
                core_num,
                mode: _mode,
                count,
            } => {
                if *core_num == 0 {
                    write!(f, "{}*{}", arch, count)
                } else {
                    write!(f, "{}({})*{}", arch, core_num, count)
                }
            }
        }
    }
}

pub struct NotDetermined;

impl From<NotDetermined> for Arch {
    fn from(_: NotDetermined) -> Self {
        Arch::Warboy
    }
}

impl From<NotDetermined> for DeviceMode {
    fn from(_: NotDetermined) -> Self {
        DeviceMode::Fusion
    }
}

impl From<NotDetermined> for u8 {
    fn from(_: NotDetermined) -> Self {
        1
    }
}

/// A builder struct for `DeviceConfig`.
pub struct DeviceConfigBuilder<A, M, C> {
    arch: A,
    mode: M,
    count: C,
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
    u8: From<C>,
{
    pub fn count(self, count: u8) -> DeviceConfig {
        let builder = DeviceConfigBuilder {
            arch: self.arch,
            mode: self.mode,
            count,
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

        DeviceConfig::Unnamed {
            arch: Arch::from(self.arch),
            core_num,
            mode,
            count: u8::from(self.count),
        }
    }
}

pub(crate) struct DeviceWithStatus {
    pub device: Device,
    pub statuses: HashMap<CoreIdx, CoreStatus>,
}

impl Deref for DeviceWithStatus {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

pub(crate) async fn expand_status(devices: Vec<Device>) -> DeviceResult<Vec<DeviceWithStatus>> {
    let mut new_devices = Vec::with_capacity(devices.len());
    for device in devices.into_iter() {
        new_devices.push(DeviceWithStatus {
            statuses: device.get_status_all().await?,
            device,
        })
    }
    Ok(new_devices)
}

pub(crate) fn find_devices_in(
    config: &DeviceConfig,
    devices: &[DeviceWithStatus],
) -> DeviceResult<Vec<DeviceFile>> {
    let mut allocated: HashMap<u8, HashSet<u8>> = HashMap::with_capacity(devices.len());

    for device in devices {
        allocated.insert(
            device.device_index(),
            device
                .statuses
                .iter()
                .filter(|(_, status)| **status != CoreStatus::Available)
                .map(|(core, _)| *core)
                .collect(),
        );
    }

    let config_count = config.count();
    let mut found: Vec<DeviceFile> = Vec::with_capacity(config_count.into());
    'outer: for _ in 0..config_count {
        for device in devices {
            'inner: for dev_file in device.dev_files() {
                if !config.fit(device.arch(), dev_file) {
                    continue 'inner;
                }

                let used = allocated.get_mut(&device.device_index()).unwrap();

                for core in used.iter() {
                    if dev_file.core_range().contains(core) {
                        continue 'inner;
                    }
                }

                // this dev_file is suitable
                found.push(dev_file.clone());
                used.extend(
                    device
                        .cores()
                        .iter()
                        .filter(|idx| dev_file.core_range().contains(idx)),
                );
                continue 'outer;
            }
        }
        return Ok(vec![]);
    }

    Ok(found)
}

#[cfg(test)]
mod tests {
    use crate::list::list_devices_with;

    use super::*;

    #[tokio::test]
    async fn test_find_devices() -> DeviceResult<()> {
        // test directory contains 2 warboy NPUs
        let devices = list_devices_with("test_data/test-0/dev", "test_data/test-0/sys").await?;
        let devices_with_statuses = expand_status(devices).await?;

        // try lookup 4 different single cores
        let config = DeviceConfig::warboy().single().count(4);
        let found = find_devices_in(&config, &devices_with_statuses)?;
        assert_eq!(found.len(), 4);
        assert_eq!(found[0].filename(), "npu0pe0");
        assert_eq!(found[1].filename(), "npu0pe1");
        assert_eq!(found[2].filename(), "npu1pe0");
        assert_eq!(found[3].filename(), "npu1pe1");

        // looking for 5 different cores should fail
        let config = DeviceConfig::warboy().single().count(5);
        let found = find_devices_in(&config, &devices_with_statuses)?;
        assert_eq!(found, vec![]);

        // try lookup 2 different fused cores
        let config = DeviceConfig::warboy().fused().count(2);
        let found = find_devices_in(&config, &devices_with_statuses)?;
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].filename(), "npu0pe0-1");
        assert_eq!(found[1].filename(), "npu1pe0-1");

        // looking for 3 different fused cores should fail
        let config = DeviceConfig::warboy().fused().count(3);
        let found = find_devices_in(&config, &devices_with_statuses)?;
        assert_eq!(found, vec![]);

        Ok(())
    }

    #[test]
    fn test_config_from_named_text_repr() -> Result<(), nom::Err<()>> {
        assert!("0:".parse::<DeviceConfig>().is_err());
        assert!(":0".parse::<DeviceConfig>().is_err());
        assert!("0:0-1-".parse::<DeviceConfig>().is_err());
        assert!("0:1-0".parse::<DeviceConfig>().is_err());

        assert_eq!(
            "0".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Named {
                device_id: 0,
                core_range: CoreRange::All
            })
        );
        assert_eq!(
            "1".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Named {
                device_id: 1,
                core_range: CoreRange::All
            })
        );
        assert_eq!(
            "0:0".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Named {
                device_id: 0,
                core_range: CoreRange::Range((0, 0))
            })
        );
        assert_eq!(
            "0:1".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Named {
                device_id: 0,
                core_range: CoreRange::Range((1, 1))
            })
        );
        assert_eq!(
            "1:1".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Named {
                device_id: 1,
                core_range: CoreRange::Range((1, 1))
            })
        );
        assert_eq!(
            "0:0-1".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Named {
                device_id: 0,
                core_range: CoreRange::Range((0, 1))
            })
        );

        Ok(())
    }

    #[test]
    fn test_config_from_unnamed_text_repr() -> Result<(), nom::Err<()>> {
        assert!("warboy".parse::<DeviceConfig>().is_err());
        assert!("warboy*".parse::<DeviceConfig>().is_err());
        assert!("*1".parse::<DeviceConfig>().is_err());
        assert!("some_npu*10".parse::<DeviceConfig>().is_err());
        assert!("warboy(2*10".parse::<DeviceConfig>().is_err());
        assert_eq!(
            "warboy(1)*2".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Unnamed {
                arch: Arch::Warboy,
                core_num: 1,
                mode: DeviceMode::Single,
                count: 2
            })
        );
        assert_eq!(
            "warboy(2)*4".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Unnamed {
                arch: Arch::Warboy,
                core_num: 2,
                mode: DeviceMode::Fusion,
                count: 4
            })
        );
        assert_eq!(
            "warboy*12".parse::<DeviceConfig>(),
            Ok(DeviceConfig::Unnamed {
                arch: Arch::Warboy,
                core_num: 0,
                mode: DeviceMode::MultiCore,
                count: 12
            })
        );
        // assert!("npu*10".parse::<DeviceConfig>().is_ok());

        Ok(())
    }

    #[test]
    fn test_config_symmetric_display() -> Result<(), nom::Err<()>> {
        assert_eq!("0".parse::<DeviceConfig>()?.to_string(), "0");
        assert_eq!("1".parse::<DeviceConfig>()?.to_string(), "1");
        assert_eq!("0:0".parse::<DeviceConfig>()?.to_string(), "0:0");
        assert_eq!("0:1".parse::<DeviceConfig>()?.to_string(), "0:1");
        assert_eq!("1:0".parse::<DeviceConfig>()?.to_string(), "1:0");
        assert_eq!("0:0-1".parse::<DeviceConfig>()?.to_string(), "0:0-1");

        assert_eq!("warboy*1".parse::<DeviceConfig>()?.to_string(), "warboy*1");
        assert_eq!(
            "warboy(1)*2".parse::<DeviceConfig>()?.to_string(),
            "warboy(1)*2"
        );
        assert_eq!(
            "warboy(2)*4".parse::<DeviceConfig>()?.to_string(),
            "warboy(2)*4"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_named_config_fit() -> DeviceResult<()> {
        let config = "0:0".parse::<DeviceConfig>().unwrap();
        let npu0pe0 = crate::get_device_with("test_data/test-0/dev", "npu0pe0").await?;
        let npu0pe1 = crate::get_device_with("test_data/test-0/dev", "npu0pe1").await?;
        let npu0pe0_1 = crate::get_device_with("test_data/test-0/dev", "npu0pe0-1").await?;
        let npu1pe0 = crate::get_device_with("test_data/test-0/dev", "npu0pe1").await?;

        assert_eq!(config.count(), 1);

        assert!(config.fit(Arch::Warboy, &npu0pe0));
        assert!(!config.fit(Arch::Warboy, &npu0pe1));
        assert!(!config.fit(Arch::Warboy, &npu0pe0_1));
        assert!(!config.fit(Arch::Warboy, &npu1pe0));

        Ok(())
    }

    #[tokio::test]
    async fn test_unnamed_config_fit() -> DeviceResult<()> {
        let config = "warboy(1)*2".parse::<DeviceConfig>().unwrap();

        assert_eq!(config.count(), 2);

        let npu0pe0 = crate::get_device_with("test_data/test-0/dev", "npu0pe0").await?;
        let npu0pe1 = crate::get_device_with("test_data/test-0/dev", "npu0pe1").await?;
        let npu0pe0_1 = crate::get_device_with("test_data/test-0/dev", "npu0pe0-1").await?;

        assert!(config.fit(Arch::Warboy, &npu0pe0));
        assert!(config.fit(Arch::Warboy, &npu0pe1));
        assert!(!config.fit(Arch::Renegade, &npu0pe0));
        assert!(!config.fit(Arch::Warboy, &npu0pe0_1));

        Ok(())
    }
}
