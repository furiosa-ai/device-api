use std::path::Path;
use tokio::fs::OpenOptions;

use crate::error::DeviceResult;

#[derive(Debug, Eq, PartialEq)]
pub enum DeviceStatus {
    Available,
    Occupied,
}

pub async fn get_device_status<P>(path: P) -> DeviceResult<DeviceStatus>
where
    P: AsRef<Path>,
{
    let res = OpenOptions::new().read(true).write(true).open(path).await;

    match res {
        Ok(_) => Ok(DeviceStatus::Available),
        Err(err) => {
            if err.raw_os_error().unwrap_or(0) == 16 {
                Ok(DeviceStatus::Occupied)
            } else {
                Err(err.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test() -> DeviceResult<()> {
        let res = get_device_status("test_data/test-0/dev/npu0").await?;
        assert_eq!(res, DeviceStatus::Available);
        Ok(())
    }
}
