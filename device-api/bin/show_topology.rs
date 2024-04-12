use cli_table::{print_stdout, Cell, Style, Table};
use furiosa_device::{list_devices, topology, DeviceError};

#[tokio::main]
async fn main() -> Result<(), DeviceError> {
    tracing_subscriber::fmt::init();

    let devices = list_devices().await?;
    if devices.is_empty() {
        println!("No devices found.");
        return Ok(());
    }

    let topology = topology::Topology::new(devices.clone())?;
    let mut rows = vec![];
    let mut header = vec!["Device".cell().bold(true)];
    for device in devices.iter() {
        let name = device.to_string();
        header.push(name.cell().bold(true));
    }
    rows.push(header);

    for device1 in devices.iter() {
        let mut row = vec![device1.to_string().cell()];
        for device2 in devices.iter() {
            let link_type = topology.get_link_type(device1, device2);
            row.push(link_type.as_ref().cell());
        }
        rows.push(row);
    }

    let table = rows.table();
    print_stdout(table)?;

    Ok(())
}
