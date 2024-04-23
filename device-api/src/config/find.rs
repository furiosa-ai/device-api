use std::collections::HashMap;
use std::ops::Deref;

use super::inner::Count;
use super::DeviceConfig;
use crate::device::{CoreIdx, CoreStatus, Device, DeviceFile};
use crate::error::DeviceResult;
use crate::{CoreRange, DeviceError};

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

    for config in &config.inner.cfgs {
        // find all device files whether available or not
        let mut fit_device_files: HashMap<DeviceFile, CoreStatus> = HashMap::new();
        for device in devices {
            for dev_file in device.dev_files() {
                if !config.fit(device.arch(), dev_file)
                    || found.iter().any(|d| d.has_intersection(dev_file))
                {
                    continue;
                }

                let cores_in_devfile = {
                    let CoreRange(start, end) = dev_file.core_range();
                    if start > end {
                        // Handle this just in case,
                        // but this is unlikely to happen since we assure that start <= end in TryFrom.
                        end..=start
                    } else {
                        start..=end
                    }
                };

                let cores_status: Vec<&CoreStatus> = cores_in_devfile
                    .map(|core_num| device.statuses.get(&core_num).unwrap())
                    .collect();

                // if there exists at least one occupied core, return the first occupied status directly
                let devfile_status = if let Some(occupied) = cores_status
                    .into_iter()
                    .find(|&status| matches!(*status, CoreStatus::Occupied(_)))
                {
                    occupied.clone()
                } else {
                    CoreStatus::Available
                };

                fit_device_files.insert(dev_file.clone(), devfile_status);
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
        let mut available_device_files: Vec<DeviceFile> = fit_device_files
            .into_iter()
            .filter(|(_, s)| *s == CoreStatus::Available)
            .map(|(d, _)| d)
            .collect();
        available_device_files.sort();

        match config.count() {
            Count::Finite(n) => match (n as usize).cmp(&available_device_files.len()) {
                std::cmp::Ordering::Less => {
                    found.extend(available_device_files.into_iter().take(n.into()))
                }
                std::cmp::Ordering::Equal => found.extend(available_device_files),
                std::cmp::Ordering::Greater => return Err(DeviceError::device_busy(config)),
            },
            Count::All => found.extend(available_device_files),
        }
    }

    Ok(found)
}
