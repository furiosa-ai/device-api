use furiosa_device_api::{list_devices, DeviceError};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), DeviceError> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry().with(env_filter).init();

    let mut found = Vec::new();
    for device in list_devices().await? {
        found.push(device);
    }

    for device in found.iter() {
        println!("{:?}", device);
    }

    Ok(())
}
