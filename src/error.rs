use std::fmt::Display;
use std::io;

use thiserror::Error;

use crate::DeviceError::{IncompatibleDriver, IoError};

pub type DeviceResult<T> = Result<T, DeviceError>;

#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("Device {name} not found")]
    DeviceNotFound { name: String },
    #[error("IoError: {cause}")]
    IoError { cause: io::Error },
    #[error("Unknown architecture: {arch}")]
    UnknownArch { arch: String },
    #[error("Incompatible device driver: {cause}")]
    IncompatibleDriver { cause: String },
}

impl DeviceError {
    pub fn file_not_found<F: Display>(file: F) -> DeviceError {
        use io::ErrorKind;
        IoError {
            cause: io::Error::new(ErrorKind::NotFound, format!("{} not found", file)),
        }
    }

    pub fn unrecognized_file<F: Display>(file: F) -> DeviceError {
        IncompatibleDriver {
            cause: format!("{} file cannot be recognized", file),
        }
    }

    pub fn invalid_device_file<F: Display>(file: F) -> DeviceError {
        IncompatibleDriver {
            cause: format!("{} is not a valid device file", file),
        }
    }
}

impl From<io::Error> for DeviceError {
    fn from(e: io::Error) -> Self {
        Self::IoError { cause: e }
    }
}
