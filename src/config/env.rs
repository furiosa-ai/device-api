use std::str::FromStr;

use super::builder::NotDetermined;
use crate::{DeviceConfig, DeviceError, DeviceResult};

#[derive(Debug)]
enum Source {
    Env(String),
    Str(String),
}

/// A struct for building `DeviceConfig` from an environment variable.
pub struct EnvBuilder<T> {
    list: Vec<Source>,
    fallback: T,
}

impl EnvBuilder<NotDetermined> {
    pub(crate) fn from_env<K: ToString>(key: K) -> Self {
        Self {
            list: vec![Source::Env(key.to_string())],
            fallback: NotDetermined { _priv: () },
        }
    }

    /// Provides a fallback env variable to the builder when the previous options are empty.
    pub fn or_env<K: ToString>(mut self, key: K) -> Self {
        self.list.push(Source::Env(key.to_string()));
        self
    }

    /// Provides a fallback option to the builder when the previous options are empty.
    /// The builder will try `from_str()` method if the item is `Some`.
    pub fn or_try<T: ToString>(mut self, item: Option<T>) -> Self {
        if let Some(s) = &item {
            self.list.push(Source::Str(s.to_string()))
        }
        self
    }

    /// Provides a fallback config to the builder when the previous options are empty.
    /// Note that incorrect syntax causes the build to fail rather than fallback.
    pub fn or(self, fallback: DeviceConfig) -> EnvBuilder<DeviceConfig> {
        EnvBuilder::<DeviceConfig> {
            list: self.list,
            fallback,
        }
    }

    /// Provides a fallback default config to the builder when the previous options are empty.
    /// Note that incorrect syntax causes the build to fail rather than fallback.
    pub fn or_default(self) -> EnvBuilder<DeviceConfig> {
        EnvBuilder::<DeviceConfig> {
            list: self.list,
            fallback: Default::default(),
        }
    }
}

impl<T: TryInto<DeviceConfig, Error: Into<DeviceError>>> EnvBuilder<T> {
    /// Finalize the config.
    pub fn build(self) -> DeviceResult<DeviceConfig> {
        for item in self.list {
            match item {
                Source::Env(ref key) => match std::env::var(key) {
                    Ok(value) => return DeviceConfig::from_str(value.as_str()),
                    Err(std::env::VarError::NotPresent) => continue,
                    Err(std::env::VarError::NotUnicode(msg)) => {
                        return Err(DeviceError::parse_error(
                            msg.to_string_lossy(),
                            format!(
                                "Environment variable '{}' value is not a valid unicode",
                                key,
                            ),
                        ))
                    }
                },
                Source::Str(item) => return DeviceConfig::from_str(item.as_str()),
            }
        }

        self.fallback.try_into().map_err(|e| e.into())
    }
}
