use std::io;

pub type DeviceResult<T> = Result<T, DeviceError>;

#[derive(Debug)]
pub enum DeviceError {
    IoError(io::Error),
    UnknownArch(String),
    UnrecognizedDeviceFile(String),
}

impl From<io::Error> for DeviceError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}
