pub use crate::arch::Arch;
pub use crate::device::{CoreStatus, Device, DeviceFile, DeviceMode};
pub use crate::error::{DeviceError, DeviceResult};
pub use crate::find::DeviceConfig;
use crate::find::{expand_status, find_devices_in};
use crate::list::list_devices_with;

mod arch;
#[cfg(feature = "blocking")]
pub mod blocking;
mod devfs;
mod device;
mod error;
mod find;
mod list;
mod status;
mod sysfs;

pub async fn list_devices() -> DeviceResult<Vec<Device>> {
    list_devices_with("/dev", "/sys").await
}

pub async fn find_devices(config: &DeviceConfig) -> DeviceResult<Vec<DeviceFile>> {
    let devices = expand_status(list_devices().await?).await?;
    find_devices_in(config, &devices)
}

/// Return a specific device if it exists.
///
/// # Arguments
///
/// * `device_name` - A device name (e.g., npu0, npu0pe0, npu0pe0-1)
pub async fn get_device<S: AsRef<str>>(device_name: S) -> DeviceResult<DeviceFile> {
    get_device_with("/dev", device_name.as_ref()).await
}

pub(crate) async fn get_device_with(devfs: &str, device_name: &str) -> DeviceResult<DeviceFile> {
    let path = devfs::path(devfs, device_name);
    if !path.exists() {
        return Err(DeviceError::DeviceNotFound {
            name: device_name.to_string(),
        });
    }

    let file = tokio::fs::File::open(&path).await?;
    if !devfs::is_character_device(file.metadata().await?.file_type()) {
        return Err(DeviceError::invalid_device_file(path.display()));
    }

    devfs::parse_indices(path.file_name().expect("not a file").to_string_lossy())?;

    DeviceFile::try_from(&path)
}
