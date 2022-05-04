use std::collections::{HashMap, HashSet};
use std::fs::FileType;
use std::io;

use std::path::{Path, PathBuf};

use crate::devfs;
use crate::devfs::is_character_device;
use tokio::fs;

use crate::device::{Device, DeviceFile, DeviceInfo};

use crate::error::DeviceResult;
use crate::sysfs::npu_mgmt::{self, BUSNAME, DEV, DEVICE_TYPE, FW_VERSION, PLATFORM_TYPE};

pub(crate) static MGMT_FILES: [(&str, bool); 4] = [
    (DEVICE_TYPE, true),
    (BUSNAME, true),
    (DEV, true),
    (FW_VERSION, false),
];

/// Allow to specify arbitrary sysfs, devfs paths for unit testing
pub(crate) async fn list_devices_with(devfs: &str, sysfs: &str) -> DeviceResult<Vec<Device>> {
    let npu_dev_files = filter_dev_files(list_devfs(devfs).await?)?;

    let mut devices: Vec<Device> = Vec::with_capacity(npu_dev_files.keys().len());

    for (idx, paths) in npu_dev_files {
        if is_furiosa_device(idx, sysfs).await {
            let mgmt_files = read_mgmt_files(sysfs, idx).await?;
            let device_info = DeviceInfo::try_from(mgmt_files)?;
            let device = collect_devices(idx, device_info, paths)?;
            devices.push(device);
        }
    }

    devices.sort();
    Ok(devices)
}

pub(crate) fn collect_devices(
    idx: u8,
    device_info: DeviceInfo,
    paths: Vec<PathBuf>,
) -> DeviceResult<Device> {
    let mut cores: HashSet<u8> = HashSet::new();
    let mut dev_files: Vec<DeviceFile> = Vec::with_capacity(paths.len());

    for path in paths {
        let file = DeviceFile::try_from(&path)?;
        cores.extend(file.core_indices());
        dev_files.push(file);
    }

    let mut cores: Vec<u8> = cores.into_iter().collect();
    cores.sort_unstable();
    dev_files.sort_by(|x, y| {
        x.core_indices()
            .len()
            .cmp(&y.core_indices().len())
            .then(x.path().cmp(y.path()))
    });

    Ok(Device::new(idx, device_info, cores, dev_files))
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
                    .or_insert_with(Vec::new)
                    .push(path.canonicalize()?); // make an absolute path
            }
        }
    }

    Ok(npu_dev_files)
}

async fn is_furiosa_device(idx: u8, sysfs: &str) -> bool {
    fs::read_to_string(npu_mgmt::path(sysfs, PLATFORM_TYPE, idx))
        .await
        .ok()
        .filter(|c| npu_mgmt::is_furiosa_platform(c))
        .is_some()
}

async fn read_mgmt_files(sysfs: &str, idx: u8) -> io::Result<HashMap<&'static str, String>> {
    let mut mgmt_files: HashMap<&'static str, String> = HashMap::new();
    for (mgmt_file, required) in MGMT_FILES {
        let path = npu_mgmt::path(sysfs, mgmt_file, idx);
        let contents = tokio::fs::read_to_string(&path)
            .await
            .or_else(|err| {
                if required {
                    Err(err)
                } else {
                    Ok(String::new())
                }
            })
            .map(|s| s.trim().to_string())?;
        if mgmt_files.insert(mgmt_file, contents).is_some() {
            unreachable!("duplicate {} file at {}", mgmt_file, path.display());
        }
    }
    Ok(mgmt_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::Arch;
    use itertools::Itertools;

    #[tokio::test]
    async fn test_find_dev_files() -> DeviceResult<()> {
        let dev_files = filter_dev_files(list_devfs("test_data/test-0/dev").await?)?;
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
            DeviceInfo::try_from(read_mgmt_files("test_data/test-0/sys", 0).await?)?.arch(),
            Arch::Warboy
        );
        assert_eq!(
            DeviceInfo::try_from(read_mgmt_files("test_data/test-0/sys", 1).await?)?.arch(),
            Arch::Warboy
        );
        Ok(())
    }
}
