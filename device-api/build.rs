use std::env;
use std::path::{Path, PathBuf};

use bindgen::Builder;
use flate2::read::GzDecoder;
use tar::Archive;

fn main() {
    let version = "2.10.0";
    let out_path = env::var("OUT_DIR").expect("No output directory given");
    let source_path = fetch_hwloc(out_path, version);
    let build_path = build_hwloc(&source_path);
    gen_hwloc_binding(&build_path);
    link_hwloc(&build_path);
}

fn fetch_hwloc(parent_path: impl AsRef<Path>, version: &str) -> PathBuf {
    let parent_path = parent_path.as_ref();
    let extracted_path = parent_path.join(format!("hwloc-{version}"));

    if extracted_path.exists() {
        eprintln!("found hwloc v{version}");
        return extracted_path;
    }

    let mut version_components = version.split('.');
    let major = version_components.next().expect("no major hwloc version");
    let minor = version_components.next().expect("no minor hwloc version");
    let url = format!(
        "https://download.open-mpi.org/release/hwloc/v{major}.{minor}/hwloc-{version}.tar.gz"
    );

    eprintln!("Downloading hwloc v{version} from URL {url}...");
    let tar_gz = attohttpc::get(url)
        .send()
        .expect("failed to GET hwloc")
        .bytes()
        .expect("failed to parse HTTP response");

    eprintln!("Extracting hwloc source...");
    let tar = GzDecoder::new(&tar_gz[..]);
    let mut archive = Archive::new(tar);
    archive
        .unpack(parent_path)
        .expect("failed to extract hwloc tar");

    // Predict location where tarball was extracted
    extracted_path
}

fn build_hwloc(source_path: &Path) -> PathBuf {
    let mut config = autotools::Config::new(source_path);
    config.enable_static().disable_shared();
    // configure to use minimalistic XML backends
    config.config_option("disable-libxml2", None);

    #[cfg(target_os = "macos")]
    config.ldflag("-F/System/Library/Frameworks -framework CoreFoundation");

    config.fast_build(true).reconf("-ivf").build()
}

fn link_hwloc(install_path: &Path) {
    let pkg_config_path = format!(
        "{}:{}",
        install_path.join("lib").join("pkgconfig").to_string_lossy(),
        install_path
            .join("lib64")
            .join("pkgconfig")
            .to_string_lossy(),
    );
    env::set_var("PKG_CONFIG_PATH", pkg_config_path);

    let pkg_config = pkg_config::Config::new();
    let found = pkg_config.probe("hwloc").expect("couldn't find a hwloc");

    for link_path in &found.link_paths {
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,{}",
            link_path
                .to_str()
                .expect("Link path is not an UTF-8 string")
        );

        println!(
            "cargo:rustc-link-search=native={}",
            link_path
                .to_str()
                .expect("Link path is not an UTF-8 string")
        );
    }

    println!("cargo:rustc-link-lib=static=hwloc");
}

fn gen_hwloc_binding(build_path: &Path) {
    let include_path = build_path.join("include");
    let hwloc_include_path = include_path.join("hwloc");
    let hwloc_autogen_include_path = hwloc_include_path.join("autogen");

    let bindings_file_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("hwloc_bindings.rs");

    Builder::default()
        .header(include_path.join("hwloc.h").to_str().unwrap())
        .clang_arg(format!("-I{}", include_path.to_str().unwrap()))
        .clang_arg(format!("-I{}", hwloc_include_path.to_str().unwrap()))
        .clang_arg(format!(
            "-I{}",
            hwloc_autogen_include_path.to_str().unwrap()
        ))
        .generate_inline_functions(true)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(bindings_file_path.clone())
        .expect("Couldn't write bindings!");

    println!(
        "hwloc bindings generated at {:?}",
        bindings_file_path.display()
    );
}
