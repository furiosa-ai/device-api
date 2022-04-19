use crate::arch::Arch;
use crate::device::{CoreStatus, Device, DeviceFile, DeviceMode};
use crate::error::DeviceResult;
use std::collections::{HashMap, HashSet};
pub struct DeviceConfig {
    arch: Arch,
    mode: DeviceMode,
    count: u8,
}

impl DeviceConfig {
    pub fn warboy() -> WarboyConfigBuilder {
        let builder = DeviceConfig {
            arch: Arch::Warboy,
            mode: DeviceMode::Single,
            count: 1,
        };
        WarboyConfigBuilder(builder)
    }
}

pub struct WarboyConfigBuilder(DeviceConfig);

impl WarboyConfigBuilder {
    pub fn raw(mut self) -> Self {
        self.0.mode = DeviceMode::Raw;
        self
    }

    pub fn fused(mut self) -> Self {
        self.0.mode = DeviceMode::Fusion;
        self
    }

    pub fn count(mut self, count: u8) -> DeviceConfig {
        self.0.count = count;
        self.0
    }

    pub fn build(self) -> DeviceConfig {
        self.0
    }
}

pub async fn find_devices_in(
    devices: &[Device],
    config: &DeviceConfig,
) -> DeviceResult<Option<Vec<DeviceFile>>> {
    let mut allocated: HashMap<u8, HashSet<u8>> = HashMap::with_capacity(devices.len());
    for device in devices {
        let status = device.get_status_all().await?;
        allocated.insert(
            device.device_index(),
            status
                .into_iter()
                .filter(|(_, status)| *status != CoreStatus::Available)
                .map(|(core, _)| core)
                .collect(),
        );
    }

    let mut found: Vec<DeviceFile> = Vec::with_capacity(config.count.into());

    'outer: for _ in 0..config.count {
        for device in devices {
            if config.arch != device.arch() {
                continue;
            }
            // early exit for raw dev
            if config.mode == DeviceMode::Raw
                && !allocated.get(&device.device_index()).unwrap().is_empty()
            {
                continue;
            }

            'inner: for dev_file in device
                .dev_files()
                .iter()
                .filter(|d| d.mode() == config.mode)
            {
                for idx in dev_file.indices() {
                    if allocated.get(&device.device_index()).unwrap().contains(idx) {
                        continue 'inner;
                    }
                }
                // this dev_file is suitable
                found.push(dev_file.clone());

                let used = allocated.get_mut(&device.device_index()).unwrap();
                used.extend(dev_file.indices());
                if dev_file.is_raw() {
                    used.extend(device.cores());
                }
                continue 'outer;
            }
        }
        return Ok(None);
    }

    Ok(Some(found))
}
