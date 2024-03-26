//! A set of synchronous APIs. This requires the optional blocking feature to be enabled.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use crate::config::find::DeviceWithStatus;
use crate::devfs::is_character_device;
use crate::device::{CoreIdx, CoreStatus, DeviceInfo};
use crate::list::{collect_devices, filter_dev_files, DevFile};
use crate::status::DeviceStatus;
use crate::sysfs::npu_mgmt;
use crate::{
    devfs, find_device_files_in, Device, DeviceConfig, DeviceError, DeviceFile, DeviceResult,
};
use crate::{hwmon, DeviceMode};

/// List all Furiosa NPU devices in the system.
pub fn list_devices() -> DeviceResult<Vec<Device>> {
    list_devices_with("/dev", "/sys")
}

/// Return a specific Furiosa NPU device in the system.
pub fn get_device(idx: u8) -> DeviceResult<Device> {
    get_device_with(idx, "/dev", "/sys")
}

/// Find a set of devices with specific configuration.
pub fn find_device_files(config: &DeviceConfig) -> DeviceResult<Vec<DeviceFile>> {
    let devices = expand_status(list_devices()?)?;
    find_device_files_in(config, &devices)
}

/// Return a specific device if it exists.
///
/// # Arguments
///
/// * `device_name` - A device name (e.g., npu0, npu0pe0, npu0pe0-1)
#[inline]
pub fn get_device_file<S: AsRef<str>>(device_name: S) -> DeviceResult<DeviceFile> {
    get_file_with("/dev", device_name.as_ref())
}

pub(crate) fn get_file_with(devfs: &str, device_name: &str) -> DeviceResult<DeviceFile> {
    let path = devfs::path(devfs, device_name);
    if !path.exists() {
        return Err(DeviceError::DeviceNotFound {
            name: device_name.to_string(),
        });
    }

    let file = File::open(&path)?;
    if !is_character_device(file.metadata()?.file_type()) {
        return Err(DeviceError::invalid_device_file(path.display()));
    }

    devfs::parse_indices(path.file_name().expect("not a file").to_string_lossy())?;

    DeviceFile::try_from(&path)
}

/// Allow to specify arbitrary sysfs, devfs paths for unit testing
pub(crate) fn list_devices_with(devfs: &str, sysfs: &str) -> DeviceResult<Vec<Device>> {
    let npu_dev_files = filter_dev_files(list_devfs(devfs)?)?;

    let mut devices: Vec<Device> = Vec::with_capacity(npu_dev_files.len());

    for (idx, paths) in npu_dev_files {
        if let Ok(device) = get_device_inner(idx, paths, devfs, sysfs) {
            devices.push(device);
        }
    }

    devices.sort();
    Ok(devices)
}

pub(crate) fn get_device_with(idx: u8, devfs: &str, sysfs: &str) -> DeviceResult<Device> {
    let mut npu_dev_files = filter_dev_files(list_devfs(devfs)?)?;

    if let Some(paths) = npu_dev_files.remove(&idx) {
        get_device_inner(idx, paths, devfs, sysfs)
    } else {
        Err(DeviceError::device_not_found(format!("npu{idx}")))
    }
}

pub(crate) fn get_device_inner(
    idx: u8,
    paths: Vec<PathBuf>,
    devfs: &str,
    sysfs: &str,
) -> DeviceResult<Device> {
    if is_furiosa_device(idx, sysfs) {
        let device_info = DeviceInfo::new(idx, PathBuf::from(devfs), PathBuf::from(sysfs));
        let busname = device_info.get(npu_mgmt::file::BUS_NAME).unwrap();
        let hwmon_fetcher = hwmon_fetcher_new(sysfs, idx, &busname)?;

        let device = collect_devices(device_info, hwmon_fetcher, paths)?;
        Ok(device)
    } else {
        Err(DeviceError::device_not_found(format!("npu{idx}")))
    }
}

fn list_devfs<P: AsRef<Path>>(devfs: P) -> io::Result<Vec<DevFile>> {
    let mut dev_files = Vec::new();

    for entry in std::fs::read_dir(devfs)? {
        let file = entry?;
        dev_files.push(DevFile {
            path: file.path(),
            file_type: file.file_type()?,
        });
    }

    Ok(dev_files)
}

fn is_furiosa_device(idx: u8, sysfs: &str) -> bool {
    std::fs::read_to_string(npu_mgmt::path_warboy(
        sysfs,
        npu_mgmt::file::PLATFORM_TYPE,
        idx,
    ))
    .ok()
    .filter(|c| npu_mgmt::is_furiosa_platform(c))
    .is_some()
}

pub(crate) fn expand_status(devices: Vec<Device>) -> DeviceResult<Vec<DeviceWithStatus>> {
    let mut new_devices = Vec::with_capacity(devices.len());
    for device in devices.into_iter() {
        new_devices.push(DeviceWithStatus {
            statuses: get_status_all(&device)?,
            device,
        })
    }
    Ok(new_devices)
}

fn get_device_status<P>(path: P) -> DeviceResult<DeviceStatus>
where
    P: AsRef<Path>,
{
    let res = OpenOptions::new().read(true).write(true).open(path);

    match res {
        Ok(_) => Ok(DeviceStatus::Available),
        Err(err) => {
            if err.raw_os_error().unwrap_or(0) == 16 {
                Ok(DeviceStatus::Occupied)
            } else {
                Err(err.into())
            }
        }
    }
}

/// Examine each core of the device, whether it is available or not.
pub fn get_status_all(device: &Device) -> DeviceResult<HashMap<CoreIdx, CoreStatus>> {
    let mut status_map = device.new_status_map();

    for file in &device.dev_files {
        if file.mode() != DeviceMode::Single {
            continue;
        }
        if get_device_status(&file.path)? == DeviceStatus::Occupied {
            for core in device
                .cores()
                .iter()
                .filter(|c| file.core_range().contains(c))
            {
                status_map.insert(*core, CoreStatus::Occupied(file.to_string()));
            }
        }
    }
    Ok(status_map)
}

fn hwmon_fetcher_new(
    base_dir: &str,
    device_index: u8,
    busname: &str,
) -> DeviceResult<hwmon::Fetcher> {
    Ok(hwmon::Fetcher {
        device_index,
        sensor_container: hwmon::SensorContainer::new_blocking(base_dir, busname).map_err(
            |cause| DeviceError::HwmonError {
                device_index,
                cause,
            },
        )?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_device_files() -> DeviceResult<()> {
        // test directory contains 2 warboy NPUs
        let devices = list_devices_with("../test_data/test-0/dev", "../test_data/test-0/sys")?;
        let devices_with_statuses = expand_status(devices)?;

        // try lookup 4 different single cores
        let config = DeviceConfig::warboy().single().count(4);
        let found_device_files = find_device_files_in(&config, &devices_with_statuses)?;
        let mut found_device_file_names: Vec<&str> =
            found_device_files.iter().map(|f| f.filename()).collect();
        found_device_file_names.sort();
        assert_eq!(
            found_device_file_names,
            &["npu0pe0", "npu0pe1", "npu1pe0", "npu1pe1"],
        );

        // looking for 5 different cores should fail
        let config = DeviceConfig::warboy().single().count(5);
        let found = find_device_files_in(&config, &devices_with_statuses);
        match found {
            Ok(_) => panic!("looking for 5 different cores should fail"),
            Err(e) => assert!(matches!(e, DeviceError::DeviceNotFound { .. })),
        }

        // // try lookup 2 different fused cores
        let config = DeviceConfig::warboy().fused().count(2);
        let found_device_files = find_device_files_in(&config, &devices_with_statuses)?;
        let mut found_device_file_names: Vec<&str> =
            found_device_files.iter().map(|f| f.filename()).collect();
        found_device_file_names.sort();
        assert_eq!(found_device_file_names, &["npu0pe0-1", "npu1pe0-1"],);

        // // looking for 3 different fused cores should fail
        let config = DeviceConfig::warboy().fused().count(3);
        let found = find_device_files_in(&config, &devices_with_statuses);
        match found {
            Ok(_) => panic!("looking for 3 different fused cores should fail"),
            Err(e) => assert!(matches!(e, DeviceError::DeviceNotFound { .. })),
        }

        Ok(())
    }

    #[test]
    fn test_get_device_file() -> DeviceResult<()> {
        get_file_with("../test_data/test-0/dev", "npu0")?;
        assert!(get_file_with("../test_data/test-0/dev", "npu0pe0").is_ok());
        assert!(get_file_with("../test_data/test-0/dev", "npu0pe1").is_ok());
        assert!(get_file_with("../test_data/test-0/dev", "npu0pe0-1").is_ok());

        assert!(matches!(
            get_file_with("../test_data/test-0/dev", "npu9"),
            Err(DeviceError::DeviceNotFound { .. })
        ));

        Ok(())
    }
}
