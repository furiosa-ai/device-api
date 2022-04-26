pub use crate::device::{Device, DeviceFile};
pub use crate::error::{DeviceError, DeviceResult};
pub use crate::find::{find_devices_in, DeviceConfig};
use crate::list::list_devices_with;

mod arch;
mod device;
mod error;
mod find;
mod list;
mod status;

pub async fn list_devices() -> DeviceResult<Vec<Device>> {
    list_devices_with("/dev", "/sys").await
}

pub async fn find_devices(config: &DeviceConfig) -> DeviceResult<Vec<DeviceFile>> {
    find_devices_in(&list_devices().await?, config).await
}
