use std::str::FromStr;

use super::builder::NotDetermined;
use crate::{DeviceConfig, DeviceError};

#[derive(Debug)]
enum Source {
    Env(String),
    Try(String),
}

struct EnvBuilder<T> {
    list: Vec<Source>,
    fallback: T,
}

impl EnvBuilder<NotDetermined> {
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

    pub fn or(mut self, fallback: DeviceConfig) -> EnvBuilder<DeviceConfig> {
        EnvBuilder::<DeviceConfig> {
            list: self.list,
            fallback,
        }
    }

    pub fn or_default(mut self) -> EnvBuilder<DeviceConfig> {
        EnvBuilder::<DeviceConfig> {
            list: self.list,
            fallback: Default::default(),
        }
    }
}

impl<T: TryInto<DeviceConfig>> EnvBuilder<T>
where
    <T as TryInto<DeviceConfig>>::Error: std::fmt::Debug,
{
    fn build(self) -> eyre::Result<DeviceConfig> {
        for item in self.list {
            match item {
                Source::Env(key) => match std::env::var(key) {
                    Ok(value) => return DeviceConfig::from_str(value.as_str()),
                    Err(std::env::VarError::NotPresent) => continue,
                    Err(err) => eyre::bail!("cause: {}", err), // TODO
                },
                Source::Try(item) => return DeviceConfig::from_str(item.as_str()),
            }
        }

        let config = self.fallback.try_into().unwrap();
        Ok(config)
    }
}
