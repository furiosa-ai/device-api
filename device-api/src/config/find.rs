use std::collections::{HashMap, HashSet};
use std::ops::Deref;

use super::inner::Count;
use super::DeviceConfig;
use crate::device::{CoreIdx, CoreStatus, Device, DeviceFile};
use crate::error::DeviceResult;
use crate::DeviceError;

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

pub(crate) fn find_device_files_in(
    config: &DeviceConfig,
    devices: &[DeviceWithStatus],
) -> DeviceResult<Vec<DeviceFile>> {
    let config = &config.inner;
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

    let mut found = Vec::new();

    for cfg in &config.cfgs {
        let count = cfg.count();
        let limit = match count {
            Count::Finite(c) => c,
            Count::All => 255, // make the loop simple by choosing a large enough number
        };
        'outer: for _ in 0..limit {
            for device in devices {
                'inner: for dev_file in device.dev_files() {
                    if !cfg.fit(device.arch(), dev_file) {
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

            // Device not found
            match count {
                Count::All => break 'outer,
                Count::Finite(_) => {
                    return Err(DeviceError::device_not_found(cfg));
                }
            }
        }
    }

    Ok(found)
}
