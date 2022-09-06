use cli_table::{print_stdout, Cell, Style, Table};

use furiosa_device::{proc, DeviceError};

#[tokio::main]
async fn main() -> Result<(), DeviceError> {
    let mut rows = vec![];

    for process in proc::scan_processes()? {
        rows.push(vec![
            process.dev_name.cell(),
            process.pid.to_string().cell(),
            process.cmdline.cell(),
        ]);
    }

    let table = rows.table().title(vec![
        "NPU".cell().bold(true),
        "PID".cell().bold(true),
        "CMD".cell().bold(true),
    ]);

    print_stdout(table)?;

    Ok(())
}
