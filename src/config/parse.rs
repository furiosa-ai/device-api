use std::fmt::Display;
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::{all_consuming, map, map_res, opt};
use nom::sequence::{delimited, preceded, separated_pair};
use nom::Parser;

use super::builder::{DeviceConfigBuilder, NotDetermined};
use crate::arch::Arch;
use crate::device::{CoreRange, DeviceFile, DeviceMode};

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
                    None | Some(1) => (1, DeviceMode::Single),
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
