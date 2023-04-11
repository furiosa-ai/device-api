use std::convert::Infallible;
use std::fmt::Display;
use std::io;

use thiserror::Error;

use crate::hwmon::error::HwmonError;
use crate::perf_regs::error::PerformanceCounterError;
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
    #[error("Unknown architecture, arch: {arch}, rev: {rev}")]
    UnknownArch { arch: String, rev: String },
    #[error("Incompatible device driver: {cause}")]
    IncompatibleDriver { cause: String },
    #[error("HwmonError: [npu{device_index}] {cause}")]
    HwmonError { device_index: u8, cause: HwmonError },
    #[error("PerformanceCounterError: {cause}")]
    PerformanceCounterError { cause: PerformanceCounterError },
    #[error("Unexpected value: {message}")]
    UnexpectedValue { message: String },
    #[error("Failed to parse given message {message}: {cause}")]
    ParseError { message: String, cause: String },
}

impl DeviceError {
    pub(crate) fn device_not_found<D: Display>(name: D) -> DeviceError {
        DeviceError::DeviceNotFound {
            name: name.to_string(),
        }
    }

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

    pub(crate) fn performance_counter_error(cause: PerformanceCounterError) -> DeviceError {
        DeviceError::PerformanceCounterError { cause }
    }

    pub(crate) fn unexpected_value<S: ToString>(message: S) -> DeviceError {
        UnexpectedValue {
            message: message.to_string(),
        }
    }

    pub(crate) fn parse_error<S: ToString, C: ToString>(message: S, cause: C) -> DeviceError {
        DeviceError::ParseError {
            message: message.to_string(),
            cause: cause.to_string(),
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

impl From<Infallible> for DeviceError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}
