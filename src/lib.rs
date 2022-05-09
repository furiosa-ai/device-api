//! A set of APIs to list and retrieve information of FuriosaAI's NPU devices.
//! For more information, see <https://furiosa.ai>.
//!
//! # Before you start
//!
//! FuriosaAI software components include kernel driver, firmware, runtime,
//! C SDK, Python SDK, and command lines tools. Currently, we offer them for
//! only users who register Early Access Program (EAP) and agree to
//! End User Licence Agreement (EULA).
//! Please contact <contact@furiosa.ai> to learn how to start the EAP.
//!
//! # Usage
//!
//! Add this to your 'Cargo.toml':
//! ```toml
//! [dependencies]
//! furiosa_device = "0.1"
//! ```
//!
//! ## Listing devices from the system
//!
//! The current implementation mainly offers two APIs, namely
//! [`list_devices`] and [`find_devices`].
//!
//! 1. [`list_devices`] enumerates all Furiosa NPU devices in the system.
//! One can simply call as below:
//! ```rust,ignore
//! let devices = list_devices().await?;
//! ```
//!
//! [Struct `Device`][`Device`] offers methods for further information of the
//! device.
//!
//! 2. If you have a desired configuration, describe it with [`DeviceConfig`]
//! and pass it to [`find_devices`]. It will lookup and return a set of
//! [`DeviceFile`]s, if available.
//! ```rust,ignore
//! // Find two Warboy devices, fused.
//! let config = DeviceConfig::warboy().fused().count(2);
//! let dev_files = find_devices(&config).await?;
//! ```
//!
//! 3. In case you have prior knowledge on the system and want to pick out a
//! device with specific name, use [`get_device`].
//! ```rust,ignore
//! let device = furiosa_device::get_device("npu0pe0").await?;
//! ```

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

/// List all Furiosa NPU devices in the system.
///
/// See the [crate-level documentation](crate).
pub async fn list_devices() -> DeviceResult<Vec<Device>> {
    list_devices_with("/dev", "/sys").await
}

/// Find a set of devices with specific configuration.
///
/// # Arguments
///
/// * `config` - DeviceConfig
///
/// See the [crate-level documentation](crate).
pub async fn find_devices(config: &DeviceConfig) -> DeviceResult<Vec<DeviceFile>> {
    let devices = expand_status(list_devices().await?).await?;
    find_devices_in(config, &devices)
}

/// Return a specific device if it exists.
///
/// # Arguments
///
/// * `device_name` - A device name (e.g., npu0, npu0pe0, npu0pe0-1)
///
/// See the [crate-level documentation](crate).
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
