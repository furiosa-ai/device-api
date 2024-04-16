use furiosa_device::{list_devices, DeviceError};

#[tokio::main]
async fn main() -> Result<(), DeviceError> {
    for device in list_devices().await? {
        let fetcher = device.get_hwmon_fetcher();

        println!("-- npu{} --", device.devfile_index());
        println!("Current");
        for sensor_value in fetcher.read_currents().await? {
            println!(
                "  {:16} {:7.2} A",
                sensor_value.label,
                f64::from(sensor_value.value) / 1000.0
            );
        }
        println!("Voltage");
        for sensor_value in fetcher.read_voltages().await? {
            println!(
                "  {:16} {:7.2} V",
                sensor_value.label,
                f64::from(sensor_value.value) / 1000.0
            );
        }
        println!("Power");
        for sensor_value in fetcher.read_powers_average().await? {
            println!(
                "  {:16} {:7.2} W",
                sensor_value.label,
                f64::from(sensor_value.value) / 1000000.0
            );
        }
        println!("Temperature");
        for sensor_value in fetcher.read_temperatures().await? {
            println!(
                "  {:16} {:7}°C",
                sensor_value.label,
                f64::from(sensor_value.value) / 1000.0
            );
        }
        println!();
    }

    Ok(())
}
