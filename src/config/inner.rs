use std::fmt::Display;
use std::str::FromStr;

use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::{all_consuming, map, map_res, opt};
use nom::sequence::{delimited, preceded, separated_pair};
use nom::Parser;

use crate::arch::Arch;
use crate::device::{CoreRange, DeviceFile, DeviceMode};

#[derive(Clone, Debug)]
pub(crate) struct DeviceConfigInner {
    pub(crate) cfgs: Vec<Config>,
}

impl FromStr for DeviceConfigInner {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            cfgs: s
                .split(',')
                .map(Config::from_str)
                .collect::<Result<Vec<_>, Self::Err>>()?,
        })
    }
}

impl Display for DeviceConfigInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cfgs.iter().map(|c| c.to_string()).join(","))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum Config {
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

impl Config {
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
                mode,
                ..
            } => arch == *config_arch && device_file.mode() == *mode,
        }
    }

    pub(crate) fn count(&self) -> u8 {
        match self {
            Self::Named { .. } => 1,
            Self::Unnamed { count, .. } => *count,
        }
    }
}

impl FromStr for Config {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn digit_to_u8<'a>(
        ) -> impl FnMut(&'a str) -> nom::IResult<&'a str, u8, nom::error::Error<&'a str>> {
            map_res(digit1, |s: &str| s.parse::<u8>())
        }
        // try parsing named configs, from patterns e.g., "0:0" or "0:0-1"
        let parsed_named = all_consuming(digit_to_u8().and(opt(preceded(
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
                Ok(Self::Named {
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
                ))(s)
                .map_err(|e| eyre::eyre!("{}", e))?;
                let (core_num, mode) = match mode {
                    None | Some(1) => (1, DeviceMode::Single),
                    // TODO: Improve below
                    Some(n) => (n, DeviceMode::Fusion),
                };

                Ok(Self::Unnamed {
                    arch,
                    core_num,
                    mode,
                    count,
                })
            }
        }
    }
}

impl Display for Config {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DeviceResult;

    #[test]
    fn test_multiple_configs_repr() -> eyre::Result<()> {
        let repr = "0:0,0:1";
        let config = repr.parse::<DeviceConfigInner>()?;

        assert_eq!(repr, config.to_string().as_str());

        Ok(())
    }

    #[tokio::test]
    async fn test_named_config_fit() -> DeviceResult<()> {
        let config = "0:0".parse::<Config>().unwrap();
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
        let config = "warboy(1)*2".parse::<Config>().unwrap();

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

    #[test]
    fn test_config_from_named_text_repr() -> Result<(), nom::Err<()>> {
        assert!("0:".parse::<Config>().is_err());
        assert!(":0".parse::<Config>().is_err());
        assert!("0:0-1-".parse::<Config>().is_err());
        assert!("0:1-0".parse::<Config>().is_err());

        assert_eq!(
            "0".parse::<Config>(),
            Ok(Config::Named {
                device_id: 0,
                core_range: CoreRange::All
            })
        );
        assert_eq!(
            "1".parse::<Config>(),
            Ok(Config::Named {
                device_id: 1,
                core_range: CoreRange::All
            })
        );
        assert_eq!(
            "0:0".parse::<Config>(),
            Ok(Config::Named {
                device_id: 0,
                core_range: CoreRange::Range((0, 0))
            })
        );
        assert_eq!(
            "0:1".parse::<Config>(),
            Ok(Config::Named {
                device_id: 0,
                core_range: CoreRange::Range((1, 1))
            })
        );
        assert_eq!(
            "1:1".parse::<Config>(),
            Ok(Config::Named {
                device_id: 1,
                core_range: CoreRange::Range((1, 1))
            })
        );
        assert_eq!(
            "0:0-1".parse::<Config>(),
            Ok(Config::Named {
                device_id: 0,
                core_range: CoreRange::Range((0, 1))
            })
        );

        Ok(())
    }

    #[test]
    fn test_config_from_unnamed_text_repr() -> Result<(), nom::Err<()>> {
        assert!("warboy".parse::<Config>().is_err());
        assert!("warboy*".parse::<Config>().is_err());
        assert!("*1".parse::<Config>().is_err());
        assert!("some_npu*10".parse::<Config>().is_err());
        assert!("warboy(2*10".parse::<Config>().is_err());
        assert_eq!(
            "warboy(1)*2".parse::<Config>(),
            Ok(Config::Unnamed {
                arch: Arch::Warboy,
                core_num: 1,
                mode: DeviceMode::Single,
                count: 2
            })
        );
        assert_eq!(
            "warboy(2)*4".parse::<Config>(),
            Ok(Config::Unnamed {
                arch: Arch::Warboy,
                core_num: 2,
                mode: DeviceMode::Fusion,
                count: 4
            })
        );
        assert_eq!(
            "warboy*12".parse::<Config>(),
            Ok(Config::Unnamed {
                arch: Arch::Warboy,
                core_num: 1,
                mode: DeviceMode::Single,
                count: 12
            })
        );
        // assert!("npu*10".parse::<Config>().is_ok());

        Ok(())
    }
}
