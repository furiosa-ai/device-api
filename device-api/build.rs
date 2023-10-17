use std::env;

use cbindgen::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::default();
    config.language = Language::C;

    Builder::new()
        .with_crate(env::var("CARGO_MANIFEST_DIR")?)
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings for device-api")
        .write_to_file("src/cbinding/include/device.h");
    Ok(())
}
