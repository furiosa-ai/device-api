use std::collections::{HashMap, HashSet};
use std::fs::FileType;
use std::io;
use std::path::{Path, PathBuf};

use tokio::fs;

use crate::devfs::{self, is_character_device};
use crate::device::{Device, DeviceFile, DeviceInfo};
use crate::error::DeviceResult;
use crate::sysfs::npu_mgmt::{self, *};
use crate::{hwmon, DeviceError};

/// Allow to specify arbitrary sysfs, devfs paths for unit testing
pub(crate) async fn list_devices_with(devfs: &str, sysfs: &str) -> DeviceResult<Vec<Device>> {
    let npu_dev_files = filter_dev_files(list_devfs(devfs).await?)?;

    let mut devices: Vec<Device> = Vec::with_capacity(npu_dev_files.keys().len());

    for (idx, paths) in npu_dev_files {
        if let Ok(device) = get_device_inner(idx, paths, devfs, sysfs).await {
            devices.push(device);
        }
    }

    devices.sort();
    Ok(devices)
}

pub(crate) async fn get_device_with(idx: u8, devfs: &str, sysfs: &str) -> DeviceResult<Device> {
    let mut npu_dev_files = filter_dev_files(list_devfs(devfs).await?)?;

    if let Some(paths) = npu_dev_files.remove(&idx) {
        get_device_inner(idx, paths, devfs, sysfs).await
    } else {
        Err(DeviceError::device_not_found(format!("npu{idx}")))
    }
}

pub(crate) async fn get_device_inner(
    idx: u8,
    paths: Vec<PathBuf>,
    devfs: &str,
    sysfs: &str,
) -> DeviceResult<Device> {
    if is_furiosa_device(idx, sysfs).await {
        let device_info = DeviceInfo::new(idx, PathBuf::from(devfs), PathBuf::from(sysfs));

        // Since busname is a required field, it is guaranteed to exist.
        let busname = device_info.get(&npu_mgmt::StaticMgmtFile::Busname).unwrap();
        let hwmon_fetcher = crate::hwmon::Fetcher::new(sysfs, idx, &busname).await?;

        let device = collect_devices(device_info, hwmon_fetcher, paths)?;
        Ok(device)
    } else {
        Err(DeviceError::device_not_found(format!("npu{idx}")))
    }
}

pub(crate) fn collect_devices(
    device_info: DeviceInfo,
    hwmon_fetcher: hwmon::Fetcher,
    paths: Vec<PathBuf>,
) -> DeviceResult<Device> {
    let mut cores: HashSet<u8> = HashSet::new();
    let mut dev_files: Vec<DeviceFile> = Vec::with_capacity(paths.len());

    for path in paths {
        let file = DeviceFile::try_from(&path)?;
        let (_, core_indices) = devfs::parse_indices(path.file_name().unwrap().to_string_lossy())?;
        cores.extend(core_indices);
        dev_files.push(file);
    }

    let mut cores: Vec<u8> = cores.into_iter().collect();
    cores.sort_unstable();
    dev_files.sort_by_key(|x| x.core_range());

    Ok(Device::new(device_info, hwmon_fetcher, cores, dev_files))
}

pub(crate) struct DevFile {
    pub path: PathBuf,
    pub file_type: FileType,
}

async fn list_devfs<P: AsRef<Path>>(devfs: P) -> io::Result<Vec<DevFile>> {
    let mut dev_files = Vec::new();

    let mut read_dir = tokio::fs::read_dir(devfs).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        dev_files.push(DevFile {
            path: entry.path(),
            file_type: entry.file_type().await?,
        });
    }

    Ok(dev_files)
}

pub(crate) fn filter_dev_files(dev_files: Vec<DevFile>) -> DeviceResult<HashMap<u8, Vec<PathBuf>>> {
    let mut npu_dev_files: HashMap<u8, Vec<PathBuf>> = HashMap::new();

    for dev_file in dev_files {
        if is_character_device(dev_file.file_type) {
            let path = &dev_file.path;
            let filename = path
                .file_name()
                .expect("No file")
                .to_string_lossy()
                .to_string();

            if let Ok((device_id, _)) = devfs::parse_indices(&filename) {
                npu_dev_files
                    .entry(device_id)
                    .or_default()
                    .push(path.canonicalize()?); // make an absolute path
            }
        }
    }

    Ok(npu_dev_files)
}

async fn is_furiosa_device(idx: u8, sysfs: &str) -> bool {
    fs::read_to_string(npu_mgmt::path(
        sysfs,
        StaticMgmtFile::PlatformType.filename(),
        idx,
    ))
    .await
    .ok()
    .filter(|c| npu_mgmt::is_furiosa_platform(c))
    .is_some()
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;
    use crate::arch::Arch;

    #[tokio::test]
    async fn test_find_dev_files() -> DeviceResult<()> {
        let dev_files = filter_dev_files(list_devfs("../test_data/test-0/dev").await?)?;
        assert_eq!(
            dev_files.keys().copied().sorted().collect::<Vec<u8>>(),
            vec![0, 1]
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_is_furiosa_device() -> tokio::io::Result<()> {
        let res = is_furiosa_device(0, "../test_data/test-0/sys").await;
        assert!(res);

        let res = is_furiosa_device(1, "../test_data/test-0/sys").await;
        assert!(res);

        let res = is_furiosa_device(2, "../test_data/test-0/sys").await;
        assert!(!res);

        Ok(())
    }

    #[test]
    fn test_identify_arch() -> DeviceResult<()> {
        let device_info = DeviceInfo::new(
            0,
            PathBuf::from("../test_data/test-0/dev"),
            PathBuf::from("../test_data/test-0/sys"),
        );
        assert_eq!(device_info.arch(), Arch::WarboyB0);
        let device_info = DeviceInfo::new(
            1,
            PathBuf::from("../test_data/test-0/dev"),
            PathBuf::from("../test_data/test-0/sys"),
        );
        assert_eq!(device_info.arch(), Arch::WarboyB0);
        Ok(())
    }
}
