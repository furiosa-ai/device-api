use std::str::FromStr;

use super::builder::NotDetermined;
use crate::{DeviceConfig, DeviceError, DeviceResult};

#[derive(Debug)]
enum Source {
    Env(String),
    Try(String),
}

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

    pub fn or_env<K: ToString>(mut self, key: K) -> Self {
        self.list.push(Source::Env(key.to_string()));
        self
    }

    pub fn or_try<T: ToString>(mut self, item: Option<T>) -> Self {
        if let Some(s) = &item {
            self.list.push(Source::Try(s.to_string()))
        }
        self
    }

    pub fn or(self, fallback: DeviceConfig) -> EnvBuilder<DeviceConfig> {
        EnvBuilder::<DeviceConfig> {
            list: self.list,
            fallback,
        }
    }

    pub fn or_default(self) -> EnvBuilder<DeviceConfig> {
        EnvBuilder::<DeviceConfig> {
            list: self.list,
            fallback: Default::default(),
        }
    }
}

impl<T: TryInto<DeviceConfig, Error = DeviceError>> EnvBuilder<T> {
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
                Source::Try(item) => return DeviceConfig::from_str(item.as_str()),
            }
        }

        let config = self.fallback.try_into().unwrap();
        Ok(config)
    }
}
