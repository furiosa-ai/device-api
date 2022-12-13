mod builder;
pub(crate) mod find;
mod inner;

use std::str::FromStr;
use std::{ffi::OsStr, fmt::Display};

pub use builder::DeviceConfigBuilder;
pub(crate) use find::{expand_status, find_devices_in};

use self::builder::NotDetermined;
use self::inner::DeviceConfigInner;
use crate::{Arch, DeviceError};

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
/// # Textual Representation
///
/// DeviceConfig supports textual representation, which is its equivalent string representation.
/// One can obtain the corresponding DeviceConfig from the textual representation
/// by using the FromStr trait, or by calling [`from_env_with_key`][`DeviceConfig::from_env_with_key`]
/// after setting an environment variable.
///
/// ```rust
/// let config = DeviceConfig::from_env()?; // default key is "FURIOSA_DEVICES"
/// let config = DeviceConfig::from_env_with_key("SOME_OTHER_ENV_KEY")?;
/// let config = DeviceConfig::from_str("0:0,0:1")?; // get config directly from a string literal
/// ```
///
/// The rules for textual representation are as follows:
///
/// ```bash
/// # Using specific device names
/// FURIOSA_DEVICES="0:0" # npu0pe0
/// FURIOSA_DEVICES="0:0-1" # npu0pe0-1

/// # Using device configs
/// FURIOSA_DEVICES="warboy*2" # warboy multi core mode x 2
/// FURIOSA_DEVICES="warboy(1)*2" # single pe x 2
/// FURIOSA_DEVICES="warboy(2)*2" # 2-pe fusioned x 2

/// # Using device configs with a random device
/// # It can be commonly used because most of systems will have a single kind of NPUs.
/// FURIOSA_DEVICES="npu(2)*2" # any 2-pe fusioned device x 2

/// # When we use multiple models in a single application
/// FURIOSA_DEVICES="APP1=warboy*2, APP2=warboy(2)*2" # Allow to specify two different device configurations for two applications 'APP1' and 'APP2'
/// ```
///
/// You can also combine multiple string representations into one DeviceConfig,
/// by separating them with commas.
///
/// ```bash
/// FURIOSA_DEVICES="0:0,0:1" # npu0pe0, npu0pe1
/// ```
#[derive(Clone, Debug)]
pub struct DeviceConfig {
    pub(crate) inner: DeviceConfigInner,
}

impl DeviceConfig {
    /// Returns a builder associated with Warboy NPUs.
    pub fn warboy() -> DeviceConfigBuilder<Arch, NotDetermined, NotDetermined> {
        DeviceConfigBuilder {
            arch: Arch::Warboy,
            mode: NotDetermined { _priv: () },
            count: NotDetermined { _priv: () },
        }
    }

    pub fn from_env_with_key<S: AsRef<OsStr>>(key: S) -> Result<Self, DeviceError> {
        match std::env::var(key) {
            Ok(message) => Ok(Self {
                inner: DeviceConfigInner::from_str(&message)
                    .map_err(|cause| DeviceError::ParseError { message, cause })?,
            }),
            Err(cause) => Err(DeviceError::EnvVarError { cause }),
        }
    }

    pub fn from_env() -> Result<Self, DeviceError> {
        Self::from_env_with_key("FURIOSA_DEVICES")
    }
}

impl Default for DeviceConfig {
    fn default() -> Self {
        DeviceConfig::warboy().fused().count(1)
    }
}

impl FromStr for DeviceConfig {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: DeviceConfigInner::from_str(s)?,
        })
    }
}

impl Display for DeviceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::list::list_devices_with;

    #[tokio::test]
    async fn test_find_devices() -> eyre::Result<()> {
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
    fn test_config_symmetric_display() -> eyre::Result<()> {
        assert_eq!("0".parse::<DeviceConfig>()?.to_string(), "0");
        assert_eq!("1".parse::<DeviceConfig>()?.to_string(), "1");
        assert_eq!("0:0".parse::<DeviceConfig>()?.to_string(), "0:0");
        assert_eq!("0:1".parse::<DeviceConfig>()?.to_string(), "0:1");
        assert_eq!("1:0".parse::<DeviceConfig>()?.to_string(), "1:0");
        assert_eq!("0:0-1".parse::<DeviceConfig>()?.to_string(), "0:0-1");

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
}
