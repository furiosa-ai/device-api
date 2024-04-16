use furiosa_device::{list_devices, DeviceError};

#[tokio::main]
async fn main() -> Result<(), DeviceError> {
    for device in list_devices().await? {
        println!("-- npu{} --", device.devfile_index());
        for frequency in device.clock_frequency()? {
            println!(
                "{:15}: {} {}",
                frequency.name(),
                frequency.value(),
                frequency.unit()
            );
        }

        println!();
    }

    Ok(())
}
