mod builder;
mod env;
pub(crate) mod find;
mod inner;

use std::fmt::Display;
use std::str::FromStr;

pub use builder::DeviceConfigBuilder;
pub(crate) use find::{expand_status, find_device_files_in};
use serde::{Deserialize, Serialize};

pub use self::builder::NotDetermined;
pub use self::env::EnvBuilder;
use self::inner::DeviceConfigInner;
use crate::{Arch, DeviceError};

/// Describes a required set of devices for [`find_device_files`][crate::find_device_files].
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
/// # Textual Representation
///
/// DeviceConfig supports textual representation, which is its equivalent string representation.
/// One can obtain the corresponding DeviceConfig from the textual representation
/// by using the FromStr trait, or by calling [`from_env`][`DeviceConfig::from_env`]
/// after setting an environment variable.
///
/// ```rust
/// use std::str::FromStr;
///
/// use furiosa_device::DeviceConfig;
///
/// let config = DeviceConfig::from_env("SOME_OTHER_ENV_KEY").build();
/// let config = DeviceConfig::from_str("npu:0:0,npu:0:1").unwrap(); // get config directly from a string literal
/// ```
///
/// The rules for textual representation are as follows:
///
/// ```rust
/// use std::str::FromStr;
///
/// use furiosa_device::DeviceConfig;
///
/// // Using specific device names
/// DeviceConfig::from_str("npu:0:0").unwrap(); // npu0pe0
/// DeviceConfig::from_str("npu:0:0-1").unwrap(); // npu0pe0-1
///
/// // Using device configs
/// DeviceConfig::from_str("warboy*2").unwrap(); // single pe x 2 (equivalent to "warboy(1)*2")
/// DeviceConfig::from_str("warboy(1)*2").unwrap(); // single pe x 2
/// DeviceConfig::from_str("warboy(2)*2").unwrap(); // 2-pe fusioned x 2
///
/// // Combine multiple representations separated by commas
/// DeviceConfig::from_str("npu:0:0-1,npu:1:0-1").unwrap(); // npu0pe0-1, npu1pe0-1
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(into = "String", try_from = "&str")]
pub struct DeviceConfig {
    pub(crate) inner: DeviceConfigInner,
}

impl DeviceConfig {
    /// Returns a builder associated with Warboy NPUs.
    pub fn warboy() -> DeviceConfigBuilder<Arch, NotDetermined, NotDetermined> {
        DeviceConfigBuilder {
            arch: Arch::WarboyB0,
            mode: NotDetermined { _priv: () },
            count: NotDetermined { _priv: () },
        }
    }

    pub fn warboy_a0() -> DeviceConfigBuilder<Arch, NotDetermined, NotDetermined> {
        DeviceConfigBuilder {
            arch: Arch::WarboyA0,
            mode: NotDetermined { _priv: () },
            count: NotDetermined { _priv: () },
        }
    }

    /// Returns a builder struct to read config saved in an environment variable.
    /// You can provide fallback options to the builder in case the envrionment variable is empty.
    pub fn from_env<K: ToString>(key: K) -> EnvBuilder<NotDetermined> {
        EnvBuilder::<NotDetermined>::from_env(key)
    }
}

impl Default for DeviceConfig {
    fn default() -> Self {
        DeviceConfig::warboy().fused().count(1)
    }
}

impl FromStr for DeviceConfig {
    type Err = DeviceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: DeviceConfigInner::from_str(s)?,
        })
    }
}

impl<'a> TryFrom<&'a str> for DeviceConfig {
    type Error = DeviceError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        DeviceConfig::from_str(value)
    }
}

impl Display for DeviceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl From<DeviceConfig> for String {
    fn from(config: DeviceConfig) -> Self {
        config.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::list::list_devices_with;

    #[tokio::test]
    async fn test_find_device_files() -> eyre::Result<()> {
        // test directory contains 2 warboy NPUs
        let devices =
            list_devices_with("../test_data/test-0/dev", "../test_data/test-0/sys").await?;
        let devices_with_statuses = expand_status(devices).await?;

        // try lookup 4 different single cores
        let config = DeviceConfig::warboy().single().count(4);
        let found_device_files = find_device_files_in(&config, &devices_with_statuses)?;
        let found_device_file_names: Vec<&str> =
            found_device_files.iter().map(|f| f.filename()).collect();
        assert_eq!(found_device_files.len(), 4);
        assert!(found_device_file_names.contains(&"npu0pe0"));
        assert!(found_device_file_names.contains(&"npu0pe1"));
        assert!(found_device_file_names.contains(&"npu1pe0"));
        assert!(found_device_file_names.contains(&"npu1pe1"));

        // try lookup all single cores
        let config = DeviceConfig::warboy().single().all();
        let found_device_files = find_device_files_in(&config, &devices_with_statuses)?;
        let found_device_file_names: Vec<&str> =
            found_device_files.iter().map(|f| f.filename()).collect();
        assert_eq!(found_device_files.len(), 4);
        assert!(found_device_file_names.contains(&"npu0pe0"));
        assert!(found_device_file_names.contains(&"npu0pe1"));
        assert!(found_device_file_names.contains(&"npu1pe0"));
        assert!(found_device_file_names.contains(&"npu1pe1"));

        // // looking for 5 different cores should fail
        let config = DeviceConfig::warboy().single().count(5);
        let found = find_device_files_in(&config, &devices_with_statuses);
        match found {
            Ok(_) => panic!("looking for 5 different cores should fail"),
            Err(e) => assert!(matches!(e, DeviceError::DeviceNotFound { .. })),
        }

        // // try lookup 2 different fused cores
        let config = DeviceConfig::warboy().fused().count(2);
        let found_device_files = find_device_files_in(&config, &devices_with_statuses)?;
        let found_device_file_names: Vec<&str> =
            found_device_files.iter().map(|f| f.filename()).collect();
        assert_eq!(found_device_files.len(), 2);
        assert!(found_device_file_names.contains(&"npu0pe0-1"));
        assert!(found_device_file_names.contains(&"npu1pe0-1"));

        // // try lookup all fused cores
        let config = DeviceConfig::warboy().fused().all();
        let found_device_files = find_device_files_in(&config, &devices_with_statuses)?;
        let found_device_file_names: Vec<&str> =
            found_device_files.iter().map(|f| f.filename()).collect();
        assert_eq!(found_device_files.len(), 2);
        assert!(found_device_file_names.contains(&"npu0pe0-1"));
        assert!(found_device_file_names.contains(&"npu1pe0-1"));

        // // looking for 3 different fused cores should fail
        let config = DeviceConfig::warboy().fused().count(3);
        let found = find_device_files_in(&config, &devices_with_statuses);
        match found {
            Ok(_) => panic!("looking for 3 different fused cores should fail"),
            Err(e) => assert!(matches!(e, DeviceError::DeviceNotFound { .. })),
        }

        Ok(())
    }

    #[test]
    fn test_config_symmetric_display() -> eyre::Result<()> {
        assert_eq!("npu:0".parse::<DeviceConfig>()?.to_string(), "npu:0");
        assert_eq!("npu:1".parse::<DeviceConfig>()?.to_string(), "npu:1");
        assert_eq!("npu:0:0".parse::<DeviceConfig>()?.to_string(), "npu:0:0");
        assert_eq!("npu:0:1".parse::<DeviceConfig>()?.to_string(), "npu:0:1");
        assert_eq!("npu:1:0".parse::<DeviceConfig>()?.to_string(), "npu:1:0");
        assert_eq!(
            "npu:0:0-1".parse::<DeviceConfig>()?.to_string(),
            "npu:0:0-1"
        );

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

    #[test]
    fn test_config_comma_separated() -> eyre::Result<()> {
        let config =
            "npu:0:0,npu:0:1,npu:0:0-1,warboy(1)*1,warboy(2)*2,npu0pe0".parse::<DeviceConfig>()?;

        assert_eq!(
            config.inner.cfgs,
            vec![
                "npu:0:0".parse::<crate::config::inner::Config>()?,
                "npu:0:1".parse::<crate::config::inner::Config>()?,
                "npu:0:0-1".parse::<crate::config::inner::Config>()?,
                "warboy(1)*1".parse::<crate::config::inner::Config>()?,
                "warboy(2)*2".parse::<crate::config::inner::Config>()?,
                "npu0pe0".parse::<crate::config::inner::Config>()?,
            ]
        );
        Ok(())
    }

    #[test]
    fn test_config_from_env() -> eyre::Result<()> {
        let key = "ENV_KEY";
        std::env::set_var(
            key,
            "npu:0:0,npu:0:1,npu:0:0-1,warboy(1)*1,warboy(2)*2,npu0pe0",
        );
        let config = DeviceConfig::from_env(key).build()?;

        assert_eq!(
            config.inner.cfgs,
            vec![
                "npu:0:0".parse::<crate::config::inner::Config>()?,
                "npu:0:1".parse::<crate::config::inner::Config>()?,
                "npu:0:0-1".parse::<crate::config::inner::Config>()?,
                "warboy(1)*1".parse::<crate::config::inner::Config>()?,
                "warboy(2)*2".parse::<crate::config::inner::Config>()?,
                "npu0pe0".parse::<crate::config::inner::Config>()?,
            ]
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_find_device_files_with_comma_separated() -> eyre::Result<()> {
        // test directory contains 2 warboy NPUs
        let devices =
            list_devices_with("../test_data/test-0/dev", "../test_data/test-0/sys").await?;
        let devices_with_statuses = expand_status(devices).await?;

        // try lookup with various valid configs
        let config = "npu:0:0,npu:0:1,npu:1:0,npu:1:1".parse::<DeviceConfig>()?;
        let found_device_files = find_device_files_in(&config, &devices_with_statuses)?;
        let found_device_file_names: Vec<&str> =
            found_device_files.iter().map(|f| f.filename()).collect();
        assert_eq!(found_device_files.len(), 4);
        assert!(found_device_file_names.contains(&"npu0pe0"));
        assert!(found_device_file_names.contains(&"npu0pe1"));
        assert!(found_device_file_names.contains(&"npu1pe0"));
        assert!(found_device_file_names.contains(&"npu1pe1"));

        let config = "npu:0:0,npu0pe1,npu:1:0,npu1pe1".parse::<DeviceConfig>()?;
        let found_device_files = find_device_files_in(&config, &devices_with_statuses)?;
        let found_device_file_names: Vec<&str> =
            found_device_files.iter().map(|f| f.filename()).collect();
        assert_eq!(found_device_files.len(), 4);
        assert!(found_device_file_names.contains(&"npu0pe0"));
        assert!(found_device_file_names.contains(&"npu0pe1"));
        assert!(found_device_file_names.contains(&"npu1pe0"));
        assert!(found_device_file_names.contains(&"npu1pe1"));

        let config = "warboy(1)*1,warboy(1)*1,warboy(1)*1,warboy(1)*1".parse::<DeviceConfig>()?;
        let found_device_files = find_device_files_in(&config, &devices_with_statuses)?;
        let found_device_file_names: Vec<&str> =
            found_device_files.iter().map(|f| f.filename()).collect();
        assert_eq!(found_device_files.len(), 4);
        assert!(found_device_file_names.contains(&"npu0pe0"));
        assert!(found_device_file_names.contains(&"npu0pe1"));
        assert!(found_device_file_names.contains(&"npu1pe0"));
        assert!(found_device_file_names.contains(&"npu1pe1"));

        let config = "npu:0:0,npu:0:1,warboy(1)*2".parse::<DeviceConfig>()?;
        let found_device_files = find_device_files_in(&config, &devices_with_statuses)?;
        let found_device_file_names: Vec<&str> =
            found_device_files.iter().map(|f| f.filename()).collect();
        assert_eq!(found_device_files.len(), 4);
        assert!(found_device_file_names.contains(&"npu0pe0"));
        assert!(found_device_file_names.contains(&"npu0pe1"));
        assert!(found_device_file_names.contains(&"npu1pe0"));
        assert!(found_device_file_names.contains(&"npu1pe1"));

        Ok(())
    }

    #[tokio::test]
    async fn test_find_device_files_with_duplicate_config() -> eyre::Result<()> {
        // test directory contains 2 warboy NPUs
        let devices =
            list_devices_with("../test_data/test-0/dev", "../test_data/test-0/sys").await?;
        let devices_with_statuses = expand_status(devices).await?;

        // test duplicate configs
        let config = "npu:0:0,npu:0:0".parse::<DeviceConfig>()?;
        let found = find_device_files_in(&config, &devices_with_statuses)?;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].filename(), "npu0pe0");

        let config = "npu:0:0-1,npu0pe0-1".parse::<DeviceConfig>()?;
        let found = find_device_files_in(&config, &devices_with_statuses)?;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].filename(), "npu0pe0-1");

        Ok(())
    }

    #[tokio::test]
    async fn test_find_device_files_with_failing_cases() -> eyre::Result<()> {
        // test directory contains 2 warboy NPUs
        let devices =
            list_devices_with("../test_data/test-0/dev", "../test_data/test-0/sys").await?;
        let devices_with_statuses = expand_status(devices).await?;

        // test trivial failing cases
        let config = "npu:2:0".parse::<DeviceConfig>()?;
        let found = find_device_files_in(&config, &devices_with_statuses);
        match found {
            Ok(_) => panic!("looking for not exist device should fail"),
            Err(e) => assert!(matches!(e, DeviceError::DeviceNotFound { .. })),
        }

        Ok(())
    }
}
