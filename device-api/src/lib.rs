//! A set of APIs to list and retrieve information of FuriosaAI's NPU devices.
//! To learn more about FuriosaAI's NPU, please visit <https://furiosa.ai>.
//!
//! # Before you start
//!
//! This crate requires FuriosaAI's NPU device and its kernel driver. Currently, FuriosaAI offers
//! NPU devices for only users who register Early Access Program (EAP). Please contact
//! <contact@furiosa.ai> to learn how to start the EAP. You can also refer to
//! [Driver, Firmware, and Runtime Installation](https://furiosa-ai.github.io/docs/latest/en/software/installation.html)
//! to learn the kernel driver installation.
//!
//! # Usage
//!
//! Add this to your 'Cargo.toml':
//! ```toml
//! [dependencies]
//! furiosa-device = "0.1"
//! ```
//!
//! ## Listing devices from the system
//!
//! The current implementation mainly offers two APIs, namely
//! [`list_devices`] and [`find_device_files`].
//!
//! 1. [`list_devices`] enumerates all Furiosa NPU devices in the system.
//! One can simply call as below:
//! ```rust,no_run
//! # #[tokio::main]
//! # async fn main() -> eyre::Result<()> {
//! let devices = furiosa_device::list_devices().await?;
//! # Ok(())
//! # }
//! ```
//!
//! [Struct `Device`][`Device`] offers methods for further information of each
//! device.
//!
//! 2. If you have a desired configuration, call [`find_device_files`] with your device configuration
//! described by a [`DeviceConfig`]. [`find_device_files`] will return a list of
//! [`DeviceFile`]s if there are matched devices.
//! ```rust,no_run
//! use furiosa_device::{find_device_files, DeviceConfig};
//!
//! // Find two Warboy devices, fused.
//! # #[tokio::main]
//! # async fn main() -> eyre::Result<()> {
//! let config = DeviceConfig::warboy().fused().count(2);
//! let dev_files = find_device_files(&config).await?;
//! # Ok(())
//! # }
//! ```
//!
//! 3. In case you have prior knowledge on the system and want to pick out a
//! device with specific name, use [`get_device_file`].
//! ```rust,no_run
//! # #[tokio::main]
//! # async fn main() -> eyre::Result<()> {
//! let device_file = furiosa_device::get_device_file("npu0pe0").await?;
//! # Ok(())
//! # }
//! ```

// Allows displaying feature flags in the documentation.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(associated_type_bounds)]

pub use crate::arch::Arch;
use crate::config::{expand_status, find_device_files_in};
pub use crate::config::{DeviceConfig, DeviceConfigBuilder, EnvBuilder, NotDetermined};
pub use crate::device::{
    ClockFrequency, CoreRange, CoreStatus, Device, DeviceFile, DeviceMode, NumaNode,
};
pub use crate::error::{DeviceError, DeviceResult};
use crate::list::{get_device_with, list_devices_with};

mod arch;
#[cfg(feature = "blocking")]
#[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
pub mod blocking;
mod config;
mod devfs;
mod device;
mod error;
pub mod hwloc;
pub mod hwmon;
mod list;
pub mod perf_regs;
pub mod proc;
mod status;
mod sysfs;

/// List all Furiosa NPU devices in the system.
///
/// See the [crate-level documentation](crate).
pub async fn list_devices() -> DeviceResult<Vec<Device>> {
    list_devices_with("/dev", "/sys").await
}

/// Return a specific Furiosa NPU device in the system.
///
/// # Arguments
///
/// * `idx` - An index number of the device (e.g., 0, 1)
///
/// See the [crate-level documentation](crate).
pub async fn get_device(idx: u8) -> DeviceResult<Device> {
    get_device_with(idx, "/dev", "/sys").await
}

/// Find a set of devices with specific configuration.
///
/// # Arguments
///
/// * `config` - DeviceConfig
///
/// See the [crate-level documentation](crate).
pub async fn find_device_files(config: &DeviceConfig) -> DeviceResult<Vec<DeviceFile>> {
    let devices = expand_status(list_devices().await?).await?;
    find_device_files_in(config, &devices)
}

/// Return a specific device if it exists.
///
/// # Arguments
///
/// * `device_name` - A device name (e.g., npu0, npu0pe0, npu0pe0-1)
///
/// See the [crate-level documentation](crate).
pub async fn get_device_file<S: AsRef<str>>(device_name: S) -> DeviceResult<DeviceFile> {
    get_device_file_with("/dev", device_name.as_ref()).await
}

pub(crate) async fn get_device_file_with(
    devfs: &str,
    device_name: &str,
) -> DeviceResult<DeviceFile> {
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
