use super::core::CoreStatus;

use tokio::fs::OpenOptions;
use std::path::Path;

pub async fn get_core_status<P>(path: P) -> CoreStatus
    where P: AsRef<Path>
{
    let res = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .await;

    match res {
        Ok(_) => CoreStatus::Available,
        Err(err) => {
            if err.raw_os_error().unwrap_or(0) == 16 {
                CoreStatus::Occupied
            } else {
                CoreStatus::Unavailable
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test() {
        let res = get_core_status("tests/test-0/dev/npu0").await;
        assert_eq!(res, CoreStatus::Available);
    }
}