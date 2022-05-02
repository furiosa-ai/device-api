pub use crate::device::{Device, DeviceFile};
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
