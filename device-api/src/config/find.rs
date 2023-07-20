use std::collections::HashMap;
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
    let mut fit_device_files: HashMap<DeviceFile, CoreStatus> = HashMap::new();
    for cfg in &config.cfgs {
        let mut found = 0;
        for device in devices {
            for dev_file in device.dev_files() {
                if !cfg.fit(device.arch(), dev_file) {
                    continue;
                }
                let core_idxes: Vec<&CoreIdx> = device
                    .cores()
                    .iter()
                    .filter(|c| dev_file.core_range.contains(c))
                    .collect();
                let status = if core_idxes
                    .iter()
                    .all(|c| *device.statuses.get(c).unwrap() == CoreStatus::Available)
                {
                    CoreStatus::Available
                } else {
                    CoreStatus::Unavailable
                };
                fit_device_files.insert(dev_file.clone(), status);
                found += 1;
            }
        }
        match cfg.count() {
            Count::Finite(n) => {
                if n > found {
                    return Err(DeviceError::device_not_found(cfg));
                }
            }
            Count::All => (),
        };
    }

    let available_device_files = fit_device_files
        .iter()
        .filter(|(_, status)| **status == CoreStatus::Available)
        .map(|(dev_file, _)| dev_file.clone())
        .collect::<Vec<DeviceFile>>();
    if available_device_files.is_empty() {
        return Err(DeviceError::device_busy(config));
    }

    Ok(available_device_files)
}
