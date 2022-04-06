use std::collections::HashMap;
use std::fs::FileType;
use std::os::unix::fs::FileTypeExt;

use std::path::PathBuf;
use std::str::FromStr;

use array_tool::vec::Intersect;
use lazy_static::lazy_static;
use regex::Regex;
use tokio::fs;

use crate::arch::Arch;
use crate::device::{Device, DeviceMode, DeviceStatus};
pub use crate::error::{DeviceError, DeviceResult};
use crate::DeviceError::UnrecognizedDeviceFile;

mod arch;
mod device;
mod error;
mod status;

lazy_static! {
    static ref REGEX_DEVICE_INDEX: Regex = Regex::new(r"^(npu)(?P<idx>\d+)pe.*$").unwrap();
}

pub async fn list_devices() -> DeviceResult<Vec<Device>> {
    list_devices_with("/dev", "/sys").await
}

/// Allow to specify arbitrary sysfs, devfs paths for unit testing
async fn list_devices_with(devfs: &str, sysfs: &str) -> DeviceResult<Vec<Device>> {
    let dev_files = find_dev_files(devfs).await?;

    let mut devices: Vec<Device> =
        Vec::with_capacity(dev_files.values().fold(0, |acc, v| acc + v.len()));
    for (idx, paths) in dev_files.into_iter() {
        if is_furiosa_device(idx, sysfs).await {
            let device_type = identify_arch(idx, sysfs).await?;
            devices.extend(collect_devices(idx, device_type, paths).await?);
        }
    }
    devices.sort();
    Ok(devices)
}

async fn collect_devices(
    idx: u8,
    device_type: Arch,
    paths: Vec<PathBuf>,
) -> DeviceResult<Vec<Device>> {
    let mut devices = Vec::with_capacity(paths.len());
    for path in paths.into_iter() {
        devices.push(recognize_device(idx, path, device_type).await?);
    }
    Ok(reconcile_devices(devices))
}

fn reconcile_devices(devices: Vec<Device>) -> Vec<Device> {
    let occupied: Vec<u8> = devices
        .iter()
        .filter(|core| core.status() == DeviceStatus::Occupied)
        .flat_map(|core| match core.mode() {
            DeviceMode::Single(idx) => vec![*idx],
            DeviceMode::Fusion(v) => v.clone(),
        })
        .collect();

    devices
        .into_iter()
        .map(|device| {
            let is_occupied = device.status() == DeviceStatus::Available
                && match device.mode() {
                    DeviceMode::Single(idx) => occupied.contains(idx),
                    DeviceMode::Fusion(indexes) => !occupied.intersect(indexes.clone()).is_empty(),
                };

            if is_occupied {
                device.change_status(DeviceStatus::Fused)
            } else {
                device
            }
        })
        .collect()
}

async fn recognize_device(device_idx: u8, dev_path: PathBuf, arch: Arch) -> DeviceResult<Device> {
    let status = status::get_device_status(&dev_path).await;

    let file_name = dev_path
        .file_name()
        .expect("not a file")
        .to_string_lossy()
        .to_string();
    DeviceMode::try_from(file_name.as_str())
        .map(|mode| Device::new(device_idx, dev_path, mode, arch, status))
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

async fn identify_arch(idx: u8, sysfs: &str) -> DeviceResult<Arch> {
    let path = format!("{}/class/npu_mgmt/npu{}_mgmt/device_type", sysfs, idx);
    let contents = fs::read_to_string(path).await?;
    Arch::from_str(contents.trim()).map_err(|_| DeviceError::UnknownArch(contents))
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[tokio::test]
    async fn test_fine_dev_files() -> DeviceResult<()> {
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
            identify_arch(0, "test_data/test-0/sys").await?,
            Arch::Warboy
        );
        assert_eq!(
            identify_arch(1, "test_data/test-0/sys").await?,
            Arch::Warboy
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_recognize_device() -> DeviceResult<()> {
        let path = PathBuf::from("test_data/test-0/dev/npu0pe0");
        let res = recognize_device(0, path, Arch::Warboy).await?;
        assert_eq!("npu0:0", res.name());
        assert_eq!(
            "test_data/test-0/dev/npu0pe0",
            res.path().as_os_str().to_string_lossy().as_ref()
        );
        assert_eq!(1, res.core_num());
        assert!(!res.fused());

        Ok(())
    }

    #[tokio::test]
    async fn test_reconcile_devices() -> tokio::io::Result<()> {
        let cores = vec![Device::new(
            0,
            PathBuf::new(),
            DeviceMode::Single(0),
            Arch::Warboy,
            DeviceStatus::Available,
        )];

        let res = reconcile_devices(cores /*, occupied*/);
        assert_eq!(res.len(), 1);
        let core0 = res.get(0).unwrap();
        assert_eq!(core0.mode(), &DeviceMode::Single(0));
        assert_eq!(core0.status(), DeviceStatus::Available);

        let cores = vec![Device::new(
            0,
            PathBuf::new(),
            DeviceMode::Single(0),
            Arch::Warboy,
            DeviceStatus::Occupied,
        )];

        let res = reconcile_devices(cores /*, occupied*/)
            .into_iter()
            .map(|c| c.status())
            .collect::<Vec<DeviceStatus>>();
        assert_eq!(res, vec![DeviceStatus::Occupied]);

        let cores = vec![
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(0),
                Arch::Warboy,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(1),
                Arch::Warboy,
                DeviceStatus::Available,
            ),
        ];

        let res = reconcile_devices(cores /*, occupied*/)
            .into_iter()
            .map(|c| c.status())
            .collect::<Vec<DeviceStatus>>();
        assert_eq!(res, vec![DeviceStatus::Available, DeviceStatus::Available]);

        let cores = vec![
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(0),
                Arch::Warboy,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(1),
                Arch::Warboy,
                DeviceStatus::Occupied,
            ),
        ];

        let res = reconcile_devices(cores /*, occupied*/)
            .into_iter()
            .map(|c| c.status())
            .collect::<Vec<DeviceStatus>>();
        assert_eq!(res, vec![DeviceStatus::Available, DeviceStatus::Occupied]);

        let cores = vec![
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(0),
                Arch::Warboy,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(1),
                Arch::Warboy,
                DeviceStatus::Occupied,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Fusion(vec![0, 1]),
                Arch::Warboy,
                DeviceStatus::Available,
            ),
        ];

        let res = reconcile_devices(cores /*, occupied*/)
            .into_iter()
            .map(|c| c.status())
            .collect::<Vec<DeviceStatus>>();
        assert_eq!(
            res,
            vec![
                DeviceStatus::Available,
                DeviceStatus::Occupied,
                DeviceStatus::Fused,
            ]
        );

        let cores = vec![
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(0),
                Arch::Warboy,
                DeviceStatus::Occupied,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(1),
                Arch::Warboy,
                DeviceStatus::Occupied,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Fusion(vec![0, 1]),
                Arch::Warboy,
                DeviceStatus::Available,
            ),
        ];

        let res = reconcile_devices(cores /*, occupied*/)
            .into_iter()
            .map(|c| c.status())
            .collect::<Vec<DeviceStatus>>();
        assert_eq!(
            res,
            vec![
                DeviceStatus::Occupied,
                DeviceStatus::Occupied,
                DeviceStatus::Fused,
            ]
        );

        let cores = vec![
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(0),
                Arch::Warboy,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(1),
                Arch::Warboy,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Fusion(vec![0, 1]),
                Arch::Warboy,
                DeviceStatus::Occupied,
            ),
        ];

        let res = reconcile_devices(cores /*, occupied*/)
            .into_iter()
            .map(|c| c.status())
            .collect::<Vec<DeviceStatus>>();
        assert_eq!(
            res,
            vec![
                DeviceStatus::Fused,
                DeviceStatus::Fused,
                DeviceStatus::Occupied,
            ]
        );

        let cores = vec![
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(0),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(1),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(2),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(3),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Fusion(vec![0, 1]),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Fusion(vec![2, 3]),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Fusion(vec![0, 1, 2, 3]),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
        ];

        let res = reconcile_devices(cores /*, occupied*/)
            .into_iter()
            .map(|c| c.status())
            .collect::<Vec<DeviceStatus>>();
        assert_eq!(
            res,
            vec![
                DeviceStatus::Available,
                DeviceStatus::Available,
                DeviceStatus::Available,
                DeviceStatus::Available,
                DeviceStatus::Available,
                DeviceStatus::Available,
                DeviceStatus::Available,
            ]
        );

        let cores = vec![
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(0),
                Arch::Renegade,
                DeviceStatus::Occupied,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(1),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(2),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(3),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Fusion(vec![0, 1]),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Fusion(vec![2, 3]),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Fusion(vec![0, 1, 2, 3]),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
        ];

        let res = reconcile_devices(cores /*, occupied*/)
            .into_iter()
            .map(|c| c.status())
            .collect::<Vec<DeviceStatus>>();
        assert_eq!(
            res,
            vec![
                DeviceStatus::Occupied,
                DeviceStatus::Available,
                DeviceStatus::Available,
                DeviceStatus::Available,
                DeviceStatus::Fused,
                DeviceStatus::Available,
                DeviceStatus::Fused,
            ]
        );

        let cores = vec![
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(0),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(1),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(2),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(3),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Fusion(vec![0, 1]),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Fusion(vec![2, 3]),
                Arch::Renegade,
                DeviceStatus::Occupied,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Fusion(vec![0, 1, 2, 3]),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
        ];

        let res = reconcile_devices(cores /*, occupied*/)
            .into_iter()
            .map(|c| c.status())
            .collect::<Vec<DeviceStatus>>();
        assert_eq!(
            res,
            vec![
                DeviceStatus::Available,
                DeviceStatus::Available,
                DeviceStatus::Fused,
                DeviceStatus::Fused,
                DeviceStatus::Available,
                DeviceStatus::Occupied,
                DeviceStatus::Fused,
            ]
        );

        let cores = vec![
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(0),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(1),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(2),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Single(3),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Fusion(vec![0, 1]),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Fusion(vec![2, 3]),
                Arch::Renegade,
                DeviceStatus::Available,
            ),
            Device::new(
                0,
                PathBuf::new(),
                DeviceMode::Fusion(vec![0, 1, 2, 3]),
                Arch::Renegade,
                DeviceStatus::Occupied,
            ),
        ];

        let res = reconcile_devices(cores /*, occupied*/)
            .into_iter()
            .map(|c| c.status())
            .collect::<Vec<DeviceStatus>>();
        assert_eq!(
            res,
            vec![
                DeviceStatus::Fused,
                DeviceStatus::Fused,
                DeviceStatus::Fused,
                DeviceStatus::Fused,
                DeviceStatus::Fused,
                DeviceStatus::Fused,
                DeviceStatus::Occupied,
            ]
        );

        Ok(())
    }
}
