#![allow(warnings)]

fn main() {
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-flags=-L /opt/homebrew/opt/hwloc/lib");
    println!("cargo:include=/opt/homebrew/opt/hwloc/include");

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-search=native=/usr/lib");
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-search=native=/usr/local/lib");

    println!("cargo:rustc-link-lib=hwloc");
}
