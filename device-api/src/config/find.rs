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
    let mut found: Vec<DeviceFile> = Vec::new();

    // sort to find named config first
    for config in &config.inner.cfgs {
        // find all device files whether available or not
        let mut fit_device_files: HashMap<DeviceFile, CoreStatus> = HashMap::new();
        for device in devices {
            for dev_file in device.dev_files() {
                if !config.fit(device.arch(), dev_file) || found.contains(dev_file) {
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
            }
        }
        match config.count() {
            Count::Finite(n) => {
                if (n as usize) > fit_device_files.len() {
                    return Err(DeviceError::device_not_found(config));
                }
            }
            Count::All => (),
        }

        // filter only available device files
        let available_device_files: Vec<DeviceFile> = fit_device_files
            .into_iter()
            .filter(|(_, s)| *s == CoreStatus::Available)
            .map(|(d, _)| d)
            .collect();
        match config.count() {
            Count::Finite(n) => match (n as usize).cmp(&available_device_files.len()) {
                std::cmp::Ordering::Less => {
                    found.extend(available_device_files.iter().take(n.into()).cloned())
                }
                std::cmp::Ordering::Equal => found.extend(available_device_files),
                std::cmp::Ordering::Greater => return Err(DeviceError::device_busy(config)),
            },
            Count::All => found.extend(available_device_files),
        }
    }

    Ok(found)
}
