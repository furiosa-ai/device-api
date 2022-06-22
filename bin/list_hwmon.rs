use furiosa_device::{list_devices, DeviceError};

#[tokio::main]
async fn main() -> Result<(), DeviceError> {
    for device in list_devices().await? {
        let accessor =
            furiosa_device::hwmon::NpuHwmonAccessor::new("/sys", device.device_index()).await?;

        println!("-- npu{} --", accessor.get_npu_idx());
        println!("Current");
        for (label, value) in accessor.read_currents().await? {
            println!("  {:16} {:7.2} A", label, f64::from(value) / 1000.0);
        }
        println!("Voltage");
        for (label, value) in accessor.read_voltages().await? {
            println!("  {:16} {:7.2} V", label, f64::from(value) / 1000.0);
        }
        println!("Power");
        for (label, value) in accessor.read_powers_average().await? {
            println!("  {:16} {:7.2} W", label, f64::from(value) / 1000000.0);
        }
        println!("Temperature");
        for (label, value) in accessor.read_temperatures().await? {
            println!("  {:16} {:7}°C", label, f64::from(value) / 1000.0);
        }
        println!();
    }

    Ok(())
}
