use super::device::DeviceStatus;

use std::path::Path;
use tokio::fs::OpenOptions;

pub async fn get_device_status<P>(path: P) -> DeviceStatus
where
    P: AsRef<Path>,
{
    let res = OpenOptions::new().read(true).write(true).open(path).await;

    match res {
        Ok(_) => DeviceStatus::Available,
        Err(err) => {
            if err.raw_os_error().unwrap_or(0) == 16 {
                DeviceStatus::Occupied
            } else {
                DeviceStatus::Unavailable
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test() {
        let res = get_device_status("test_data/test-0/dev/npu0").await;
        assert_eq!(res, DeviceStatus::Available);
    }
}
