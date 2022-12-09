use std::collections::{HashMap, HashSet};
use std::ops::Deref;

use super::DeviceConfig;
use crate::device::{CoreIdx, CoreStatus, Device, DeviceFile};
use crate::error::DeviceResult;

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
    let config = config.inner;
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
