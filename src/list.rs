use std::collections::{HashMap, HashSet};
use std::fs::FileType;
use std::os::unix::fs::FileTypeExt;

use std::path::PathBuf;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;
use tokio::fs;

use crate::arch::Arch;
use crate::device::{Device, DeviceFile, DeviceInfo};

use crate::error::{DeviceError, DeviceResult};
use crate::DeviceError::UnrecognizedDeviceFile;

lazy_static! {
    static ref REGEX_DEVICE_INDEX: Regex = Regex::new(r"^(npu)(?P<idx>\d+)($|pe.*$)").unwrap();
}

/// Allow to specify arbitrary sysfs, devfs paths for unit testing
pub(crate) async fn list_devices_with(devfs: &str, sysfs: &str) -> DeviceResult<Vec<Device>> {
    let dev_files = find_dev_files(devfs).await?;

    let mut devices: Vec<Device> = Vec::with_capacity(dev_files.keys().len());

    for (idx, paths) in dev_files.into_iter() {
        if is_furiosa_device(idx, sysfs).await {
            let device_info = identify_device(idx, sysfs).await?;
            let device = collect_devices(idx, device_info, paths)?;
            devices.push(device);
        }
    }

    devices.sort();
    Ok(devices)
}

fn collect_devices(idx: u8, device_info: DeviceInfo, paths: Vec<PathBuf>) -> DeviceResult<Device> {
    let mut cores: HashSet<u8> = HashSet::new();
    let mut dev_files: Vec<DeviceFile> = Vec::with_capacity(paths.len());

    for path in paths {
        let file = DeviceFile::try_from(&path)?;
        cores.extend(file.indices());
        dev_files.push(file);
    }

    let mut cores: Vec<u8> = cores.into_iter().collect();
    cores.sort_unstable();
    dev_files.sort();
    Ok(Device::new(idx, device_info, cores, dev_files))
}

fn is_character_device(file_type: FileType) -> bool {
    if cfg!(test) {
        file_type.is_file()
    } else {
        file_type.is_char_device()
    }
}

async fn find_dev_files(devfs: &str) -> DeviceResult<HashMap<u8, Vec<PathBuf>>> {
    let mut dev_files: HashMap<u8, Vec<PathBuf>> = HashMap::new();

    let mut entries = fs::read_dir(devfs).await?;
    while let Some(entry) = entries.next_entry().await? {
        if is_character_device(entry.file_type().await?) {
            // allow just a file too for unit testing
            let filename = entry.file_name().to_string_lossy().to_string();
            if let Some(x) = REGEX_DEVICE_INDEX.captures(&filename) {
                let idx: u8 = x
                    .name("idx")
                    .ok_or_else(|| UnrecognizedDeviceFile(filename.clone()))?
                    .as_str()
                    .parse()
                    .map_err(|_| UnrecognizedDeviceFile(filename))?;
                // make an absolute path
                let absolute_path = std::fs::canonicalize(entry.path())?;
                dev_files
                    .entry(idx)
                    .or_insert_with(Vec::new)
                    .push(absolute_path);
            }
        }
    }

    Ok(dev_files)
}

async fn is_furiosa_device(idx: u8, sysfs: &str) -> bool {
    let path = format!("{}/class/npu_mgmt/npu{}_mgmt/platform_type", sysfs, idx);

    fs::read_to_string(path)
        .await
        .ok()
        .filter(|s| {
            let platform = s.trim();
            // FuriosaAI in Warboy, VITIS in U250
            platform == "FuriosaAI" || platform == "VITIS"
        })
        .is_some()
}

async fn identify_device(idx: u8, sysfs: &str) -> DeviceResult<DeviceInfo> {
    let path = format!("{}/class/npu_mgmt/npu{}_mgmt/device_type", sysfs, idx);
    let contents = fs::read_to_string(path).await?;
    let arch = Arch::from_str(contents.trim()).map_err(|_| DeviceError::UnknownArch(contents))?;

    let path = format!("{}/class/npu_mgmt/npu{}_mgmt/busname", sysfs, idx);
    let busname = fs::read_to_string(path)
        .await
        .and_then(|s| Ok(String::from(s.trim())))
        .ok();

    let path = format!("{}/class/npu_mgmt/npu{}_mgmt/dev", sysfs, idx);
    let pci_dev = fs::read_to_string(path)
        .await
        .and_then(|s| Ok(String::from(s.trim())))
        .ok();

    let path = format!("{}/class/npu_mgmt/npu{}_mgmt/fw_version", sysfs, idx);
    let firmware_version = fs::read_to_string(path)
        .await
        .and_then(|s| Ok(String::from(s.trim())))
        .ok();

    Ok(DeviceInfo::new(arch, busname, pci_dev, firmware_version))
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[tokio::test]
    async fn test_find_dev_files() -> DeviceResult<()> {
        let dev_files = find_dev_files("test_data/test-0/dev").await?;
        assert_eq!(
            dev_files.keys().copied().sorted().collect::<Vec<u8>>(),
            vec![0, 1]
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_is_furiosa_device() -> tokio::io::Result<()> {
        let res = is_furiosa_device(0, "test_data/test-0/sys").await;
        assert!(res);

        let res = is_furiosa_device(1, "test_data/test-0/sys").await;
        assert!(res);

        let res = is_furiosa_device(2, "test_data/test-0/sys").await;
        assert!(!res);

        Ok(())
    }

    #[tokio::test]
    async fn test_identify_arch() -> DeviceResult<()> {
        assert_eq!(
            identify_device(0, "test_data/test-0/sys").await?.arch(),
            Arch::Warboy
        );
        assert_eq!(
            identify_device(1, "test_data/test-0/sys").await?.arch(),
            Arch::Warboy
        );
        Ok(())
    }
}
