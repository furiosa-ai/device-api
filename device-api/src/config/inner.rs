use std::cmp::Ordering;
use std::fmt::Display;
use std::str::FromStr;

use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::{all_consuming, map, map_res, opt};
use nom::sequence::{delimited, preceded, separated_pair, tuple};

use crate::arch::Arch;
use crate::device::{CoreRange, DeviceFile};
use crate::{DeviceError, DeviceResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum Count {
    Finite(u8),
    All,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct Cores(pub(crate) u8);

impl Display for Count {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Count::Finite(c) => write!(f, "{c}"),
            // TODO: revise syntax below and bring according implementation to Config's FromStr
            Count::All => write!(f, "all"),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DeviceConfigInner {
    pub(crate) cfgs: Vec<Config>,
}

impl FromStr for DeviceConfigInner {
    type Err = DeviceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cfgs = s
            .split(',')
            .map(Config::from_str)
            .collect::<Result<Vec<_>, Self::Err>>()?;
        // sort to find named config first
        cfgs.sort();
        Ok(Self { cfgs })
    }
}

impl Display for DeviceConfigInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cfgs.iter().map(|c| c.to_string()).join(","))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum Config {
    Named {
        arch: Arch,
        devfile_index: u8,
        core_range: CoreRange,
    },
    Unnamed {
        arch: Arch,
        core_num: u8,
        count: Count,
    },
}

impl Config {
    pub(crate) fn fit(&self, arch: Arch, device_file: &DeviceFile) -> bool {
        match self {
            Self::Named {
                arch: config_arch,
                devfile_index: device_id,
                core_range,
            } => {
                arch == *config_arch
                    && device_file.devfile_index() == *device_id
                    && device_file.core_range() == *core_range
            }
            Self::Unnamed {
                arch: config_arch,
                core_num: config_core_num,
                ..
            } => {
                let CoreRange(start, end) = device_file.core_range;
                arch == *config_arch && (end - start + 1) == *config_core_num
            }
        }
    }

    pub(crate) fn count(&self) -> Count {
        match self {
            Self::Named { .. } => Count::Finite(1),
            Self::Unnamed { count, .. } => *count,
        }
    }
}

impl PartialOrd for Config {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Config {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Config::Named { .. }, Config::Unnamed { .. }) => Ordering::Less,
            (Config::Unnamed { .. }, Config::Named { .. }) => Ordering::Greater,
            _ => Ordering::Equal,
        }
    }
}

impl FromStr for Config {
    type Err = DeviceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_arch<'a>(
        ) -> impl FnMut(&'a str) -> nom::IResult<&'a str, Arch, nom::error::Error<&'a str>>
        {
            // TODO: rngd will be added later
            let p = alt((tag("npu"), tag("warboy")));
            map_res(p, |s: &str| match s {
                "npu" | "warboy" => Ok(Arch::WarboyB0),
                _ => Err(format!("Invalid architecture: {}", s)),
            })
        }

        fn digit_to_u8<'a>(
        ) -> impl FnMut(&'a str) -> nom::IResult<&'a str, u8, nom::error::Error<&'a str>> {
            map_res(digit1, |s: &str| s.parse::<u8>())
        }

        fn parse_cores<'a>(
        ) -> impl FnMut(&'a str) -> nom::IResult<&'a str, CoreRange, nom::error::Error<&'a str>>
        {
            alt((
                map_res(
                    separated_pair(digit_to_u8(), tag("-"), digit_to_u8()),
                    CoreRange::try_from,
                ),
                map(digit_to_u8(), CoreRange::from),
            ))
        }

        // Try parsing a "0:0" or "0:0-1" pattern.
        fn named_cfg_parser(s: &str) -> DeviceResult<Config> {
            let parser_arch = parse_arch();
            let parser_id = preceded(tag(":"), digit_to_u8());
            let parser_cores = preceded(tag(":"), parse_cores());
            let mut parser = all_consuming(tuple((parser_arch, parser_id, parser_cores)));

            let (_, (arch, devfile_index, core_range)) =
                parser(s).map_err(|e| DeviceError::parse_error(s, e.to_string()))?;

            Ok(Config::Named {
                arch,
                devfile_index,
                core_range,
            })
        }

        // Try parsing a "warboy(1)*1" pattern
        fn unnamed_cfg_parser(s: &str) -> DeviceResult<Config> {
            // TODO: rngd will be added later
            let parser_arch = map_res(tag("warboy"), |s: &str| s.parse::<Arch>());
            let parser_cores = map(opt(delimited(tag("("), digit_to_u8(), tag(")"))), |c| {
                c.unwrap_or(1)
            });
            let parser_count = preceded(tag("*"), digit_to_u8());
            let mut parser = all_consuming(tuple((parser_arch, parser_cores, parser_count)));

            // Note: nom::sequence::tuple requires parsers to have equivalent signatures
            let (_, (arch, core_num, count)) =
                parser(s).map_err(|e| DeviceError::parse_error(s, e.to_string()))?;

            Ok(Config::Unnamed {
                arch,
                core_num,
                count: Count::Finite(count),
            })
        }

        named_cfg_parser(s).or_else(|_| unnamed_cfg_parser(s))
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Named {
                arch,
                devfile_index: device_id,
                core_range,
            } => {
                let CoreRange(start, end) = core_range;
                if start == end {
                    write!(f, "{arch}:{device_id}:{start}")
                } else {
                    write!(f, "{arch}:{device_id}:{start}-{end}")
                }
            }
            Self::Unnamed {
                arch,
                core_num,
                count,
            } => {
                if *core_num == 0 {
                    write!(f, "{arch}*{count}")
                } else {
                    write!(f, "{arch}({core_num})*{count}")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_configs_repr() -> eyre::Result<()> {
        let repr = "warboy:0:0,warboy:0:1";
        let config = repr.parse::<DeviceConfigInner>()?;

        assert_eq!(repr, config.to_string().as_str());

        Ok(())
    }

    #[tokio::test]
    async fn test_named_config_fit() -> DeviceResult<()> {
        let config = "npu:0:0".parse::<Config>().unwrap();
        let npu0pe0 = crate::get_device_file_with("../test_data/test-0/dev", "npu0pe0").await?;
        let npu0pe1 = crate::get_device_file_with("../test_data/test-0/dev", "npu0pe1").await?;
        let npu0pe0_1 = crate::get_device_file_with("../test_data/test-0/dev", "npu0pe0-1").await?;
        let npu1pe0 = crate::get_device_file_with("../test_data/test-0/dev", "npu0pe1").await?;

        assert_eq!(config.count(), Count::Finite(1));

        assert!(config.fit(Arch::WarboyB0, &npu0pe0));
        assert!(!config.fit(Arch::WarboyB0, &npu0pe1));
        assert!(!config.fit(Arch::WarboyB0, &npu0pe0_1));
        assert!(!config.fit(Arch::WarboyB0, &npu1pe0));

        Ok(())
    }

    #[tokio::test]
    async fn test_unnamed_config_fit() -> DeviceResult<()> {
        let config = "warboy(1)*2".parse::<Config>().unwrap();

        assert_eq!(config.count(), Count::Finite(2));

        let npu0pe0 = crate::get_device_file_with("../test_data/test-0/dev", "npu0pe0").await?;
        let npu0pe1 = crate::get_device_file_with("../test_data/test-0/dev", "npu0pe1").await?;
        let npu0pe0_1 = crate::get_device_file_with("../test_data/test-0/dev", "npu0pe0-1").await?;

        assert!(config.fit(Arch::WarboyB0, &npu0pe0));
        assert!(config.fit(Arch::WarboyB0, &npu0pe1));
        assert!(!config.fit(Arch::RNGD, &npu0pe0));
        assert!(!config.fit(Arch::WarboyB0, &npu0pe0_1));

        Ok(())
    }

    #[test]
    fn test_config_from_named_text_repr() -> eyre::Result<()> {
        assert!("0".parse::<Config>().is_err());
        assert!("npu0:".parse::<Config>().is_err());
        assert!("npu:0:0-1-".parse::<Config>().is_err());
        assert!("npu:0:1-0".parse::<Config>().is_err());
        assert!("npu:0".parse::<Config>().is_err());

        assert_eq!(
            "npu:0:0".parse::<Config>()?,
            Config::Named {
                arch: Arch::WarboyB0,
                devfile_index: 0,
                core_range: CoreRange(0, 0)
            }
        );
        assert_eq!(
            "npu:0:1".parse::<Config>()?,
            Config::Named {
                arch: Arch::WarboyB0,
                devfile_index: 0,
                core_range: CoreRange(1, 1)
            }
        );
        assert_eq!(
            "npu:1:1".parse::<Config>()?,
            Config::Named {
                arch: Arch::WarboyB0,
                devfile_index: 1,
                core_range: CoreRange(1, 1)
            }
        );
        assert_eq!(
            "npu:0:0-1".parse::<Config>()?,
            Config::Named {
                arch: Arch::WarboyB0,
                devfile_index: 0,
                core_range: CoreRange(0, 1)
            }
        );

        Ok(())
    }

    #[test]
    fn test_config_from_unnamed_text_repr() -> eyre::Result<()> {
        assert!("warboy".parse::<Config>().is_err());
        assert!("warboy*".parse::<Config>().is_err());
        assert!("*1".parse::<Config>().is_err());
        assert!("some_npu*10".parse::<Config>().is_err());
        assert!("warboy(2*10".parse::<Config>().is_err());
        assert_eq!(
            "warboy(1)*2".parse::<Config>()?,
            Config::Unnamed {
                arch: Arch::WarboyB0,
                core_num: 1,
                count: Count::Finite(2),
            }
        );
        assert_eq!(
            "warboy(2)*4".parse::<Config>()?,
            Config::Unnamed {
                arch: Arch::WarboyB0,
                core_num: 2,
                count: Count::Finite(4)
            }
        );
        assert_eq!(
            "warboy*12".parse::<Config>()?,
            Config::Unnamed {
                arch: Arch::WarboyB0,
                core_num: 1,
                count: Count::Finite(12)
            }
        );

        Ok(())
    }
}
