use cli_table::{print_stdout, Cell, Style, Table};
use furiosa_device_api::{list_devices, DeviceError};
use itertools::join;
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

    let mut rows = vec![];

    for device in found.iter() {
        let status = device.get_status_all().await?;
        let mut status: Vec<(u8, _)> = status.into_iter().collect();
        status.sort_by(|a, b| a.0.cmp(&b.0));
        let msg = join(
            status
                .iter()
                .map(|(k, v)| k.to_string() + " (" + &v.to_string() + ")"),
            ", ",
        );
        rows.push(vec![device.to_string().cell(), msg.cell()]);
    }
    let table = rows
        .table()
        .title(vec!["NPU".cell().bold(true), "Cores".cell().bold(true)]);
    print_stdout(table)?;

    Ok(())
}
