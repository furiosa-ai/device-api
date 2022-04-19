use crate::device::Device;
pub use crate::error::{DeviceError, DeviceResult};
use crate::list::list_devices_with;

mod arch;
mod device;
mod error;
mod list;
mod status;

pub async fn list_devices() -> DeviceResult<Vec<Device>> {
    list_devices_with("/dev", "/sys").await
}
