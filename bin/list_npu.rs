use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use furiosa_device_api::{list_devices, DeviceError};

#[tokio::main]
async fn main() -> Result<(), DeviceError> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry().with(env_filter).init();

    let mut found = Vec::new();
    // find 2 pes
    for device in list_devices().await? {
        eprintln!("{}", device);
        if device.available() && device.single_core() {
            found.push(device);
        }
    }

    for device in found.iter() {
        println!("{}", device.path().display());
    }

    Ok(())
}
