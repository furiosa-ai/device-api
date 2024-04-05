use std::collections::{HashMap, HashSet};
use std::fs::FileType;
use std::io;
use std::path::{Path, PathBuf};

use strum::IntoEnumIterator;
use tokio::fs;

use crate::arch::Arch;
use crate::devfs::{self, is_character_device};
use crate::device::{Device, DeviceFile, DeviceInner};
use crate::error::DeviceResult;
use crate::sysfs::npu_mgmt;
use crate::{hwmon, DeviceError};

/// Allow to specify arbitrary sysfs, devfs paths for unit testing
pub(crate) async fn list_devices_with(devfs: &str, sysfs: &str) -> DeviceResult<Vec<Device>> {
    let mut devices = Vec::new();
    for arch in Arch::iter() {
        let d = list_devices_with_arch(arch, devfs, sysfs).await?;
        devices.extend(d);
    }
    Ok(devices)
}

pub async fn list_devices_with_arch(
    arch: Arch,
    devfs: &str,
    sysfs: &str,
) -> DeviceResult<Vec<Device>> {
    let dev_files = filter_dev_files(list_dev_files(arch, devfs).await?)?;
    let mut devices: Vec<Device> = Vec::with_capacity(dev_files.keys().len());
    for (idx, paths) in dev_files {
        if let Ok(device) = get_device_inner(arch, idx, paths, devfs, sysfs).await {
            devices.push(device);
        }
    }

    devices.sort();
    Ok(devices)
}

/// Deprecated: idx no longer unique
pub(crate) async fn get_device_with(
    arch: Arch,
    idx: u8,
    devfs: &str,
    sysfs: &str,
) -> DeviceResult<Device> {
    let mut npu_dev_files = filter_dev_files(list_dev_files(arch, devfs).await?)?;
    if let Some(paths) = npu_dev_files.remove(&idx) {
        get_device_inner(arch, idx, paths, devfs, sysfs).await
    } else {
        Err(DeviceError::device_not_found(format!("npu{idx}")))
    }
}

pub(crate) async fn get_device_inner(
    arch: Arch,
    idx: u8,
    paths: Vec<PathBuf>,
    devfs: &str,
    sysfs: &str,
) -> DeviceResult<Device> {
    if is_furiosa_device(arch, idx, sysfs).await {
        let inner = arch.create_inner(idx, devfs, sysfs)?;
        let busname = inner.busname();
        let hwmon_fetcher = crate::hwmon::Fetcher::new(sysfs, idx, &busname).await?;

        let device = collect_devices(inner, hwmon_fetcher, paths)?;
        Ok(device)
    } else {
        Err(DeviceError::device_not_found(format!("npu{idx}")))
    }
}

pub(crate) fn collect_devices(
    inner: Box<dyn DeviceInner>,
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

    Ok(Device::new(inner, hwmon_fetcher, cores, dev_files))
}

pub(crate) struct DevFile {
    pub path: PathBuf,
    pub file_type: FileType,
}

/// List all files in the devfs directory, including /dev/renegade/.
async fn list_dev_files<P: AsRef<Path>>(arch: Arch, path: P) -> io::Result<Vec<DevFile>> {
    let mut dev_files = Vec::new();
    let path = arch.devfile_path(path);
    let mut read_dir = match tokio::fs::read_dir(path).await {
        Ok(rd) => rd,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(dev_files),
        Err(e) => return Err(e),
    };
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

async fn is_furiosa_device(arch: Arch, idx: u8, sysfs: &str) -> bool {
    fs::read_to_string(arch.platform_type_path(idx, sysfs))
        .await
        .ok()
        .filter(|c| npu_mgmt::is_furiosa_platform(c))
        .is_some()
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    fn sorted_keys<K, V>(map: &HashMap<K, V>) -> Vec<K>
    where
        K: Ord + Copy,
    {
        map.keys().copied().sorted().collect()
    }

    #[tokio::test]
    async fn test_find_dev_files() -> DeviceResult<()> {
        let dev_files =
            filter_dev_files(list_dev_files(Arch::WarboyB0, "../test_data/test-0/dev").await?)?;
        assert_eq!(sorted_keys(&dev_files), vec![0, 1]);

        let dev_files =
            filter_dev_files(list_dev_files(Arch::Renegade, "../test_data/test-0/dev").await?)?;
        assert_eq!(sorted_keys(&dev_files), vec![] as Vec<u8>);

        let dev_files =
            filter_dev_files(list_dev_files(Arch::WarboyB0, "../test_data/test-1/dev/").await?)?;
        assert_eq!(sorted_keys(&dev_files), vec![0]);

        let dev_files =
            filter_dev_files(list_dev_files(Arch::Renegade, "../test_data/test-1/dev/").await?)?;
        assert_eq!(sorted_keys(&dev_files), vec![0]);

        Ok(())
    }

    #[tokio::test]
    async fn test_is_furiosa_device() -> tokio::io::Result<()> {
        // only two warboy devices (0, 1) in test-0
        let res = is_furiosa_device(Arch::WarboyB0, 0, "../test_data/test-0/sys").await;
        assert!(res);

        let res = is_furiosa_device(Arch::WarboyB0, 1, "../test_data/test-0/sys").await;
        assert!(res);

        let res = is_furiosa_device(Arch::WarboyB0, 2, "../test_data/test-0/sys").await;
        assert!(!res);

        let res = is_furiosa_device(Arch::Renegade, 0, "../test_data/test-0/sys").await;
        assert!(!res);

        // one warboy, one renegade device, both have index 0, in test-1
        let res = is_furiosa_device(Arch::WarboyB0, 0, "../test_data/test-1/sys").await;
        assert!(res);

        let res = is_furiosa_device(Arch::Renegade, 0, "../test_data/test-1/sys").await;
        assert!(res);

        let res = is_furiosa_device(Arch::WarboyB0, 1, "../test_data/test-1/sys").await;
        assert!(!res);

        let res = is_furiosa_device(Arch::Renegade, 1, "../test_data/test-1/sys").await;
        assert!(!res);

        Ok(())
    }
}
