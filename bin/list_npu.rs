use furiosa_device_api::list_devices;

#[tokio::main]
async fn main() {
    let mut found = Vec::new();
    // find 2 pes
    for device in list_devices().await {
        eprintln!("{}", device);
        if device.available() && device.single_core() {
            found.push(device);
        }
    }

    for device in found.iter() {
        println!("{}", device.path().display());
    }
}
