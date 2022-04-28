use std::fmt::Display;
use std::io;

use thiserror::Error;

use crate::DeviceError::IncompatibleDriver;

pub type DeviceResult<T> = Result<T, DeviceError>;

#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("IoError: {cause}")]
    IoError { cause: io::Error },
    #[error("Unknown architecture: {arch}")]
    UnknownArch { arch: String },
    #[error("Incompatible device driver: {cause}")]
    IncompatibleDriver { cause: String },
}

impl DeviceError {
    pub fn unrecognized_file<F: Display>(file: F) -> DeviceError {
        IncompatibleDriver {
            cause: format!("{} file cannot be recognized", file),
        }
    }
}

impl From<io::Error> for DeviceError {
    fn from(e: io::Error) -> Self {
        Self::IoError { cause: e }
    }
}
