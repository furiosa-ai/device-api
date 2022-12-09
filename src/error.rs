use std::fmt::Display;
use std::io;

use thiserror::Error;

use crate::hwmon::error::HwmonError;
use crate::DeviceError::{IncompatibleDriver, IoError, UnexpectedValue};

/// Type alias for `Result<T, DeviceError>`.
pub type DeviceResult<T> = Result<T, DeviceError>;

/// An error that occurred during parsing or retrieving devices.
#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("Device {name} not found")]
    DeviceNotFound { name: String },
    #[error("IoError: {cause}")]
    IoError { cause: io::Error },
    #[error("PermissionDenied: {cause}")]
    PermissionDenied { cause: io::Error },
    #[error("Unknown architecture: {arch}")]
    UnknownArch { arch: String },
    #[error("Incompatible device driver: {cause}")]
    IncompatibleDriver { cause: String },
    #[error("HwmonError: [npu{device_index}] {cause}")]
    HwmonError { device_index: u8, cause: HwmonError },
    #[error("Unexpected value: {message}")]
    UnexpectedValue { message: String },
    #[error("Failed to parse given message {message}: {cause}")]
    ParseError {
        message: String,
        cause: eyre::Error,
    },
    #[error("Coud not retrieve the environment variable")]
    EnvVarError { cause: std::env::VarError },
}

impl DeviceError {
    pub(crate) fn file_not_found<F: Display>(file: F) -> DeviceError {
        use io::ErrorKind;
        IoError {
            cause: io::Error::new(ErrorKind::NotFound, format!("{} not found", file)),
        }
    }

    pub(crate) fn unrecognized_file<F: Display>(file: F) -> DeviceError {
        IncompatibleDriver {
            cause: format!("{} file cannot be recognized", file),
        }
    }

    pub(crate) fn invalid_device_file<F: Display>(file: F) -> DeviceError {
        IncompatibleDriver {
            cause: format!("{} is not a valid device file", file),
        }
    }

    pub(crate) fn unsupported_key<K: Display>(key: K) -> DeviceError {
        IncompatibleDriver {
            cause: format!("mgmt file {} is not supported", key),
        }
    }

    pub(crate) fn hwmon_error(device_index: u8, cause: HwmonError) -> DeviceError {
        DeviceError::HwmonError {
            device_index,
            cause,
        }
    }

    pub(crate) fn unexpected_value<S: ToString>(message: S) -> DeviceError {
        UnexpectedValue {
            message: message.to_string(),
        }
    }
}

impl From<io::Error> for DeviceError {
    fn from(e: io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            Self::PermissionDenied { cause: e }
        } else {
            Self::IoError { cause: e }
        }
    }
}
