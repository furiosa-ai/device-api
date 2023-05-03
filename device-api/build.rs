#![warn(rust_2018_idioms)]

use std::env::{self, VarError};
use std::str;

use chrono::offset::Utc;
use git2::{Repository, Status, StatusOptions};

/// Sets the build metadata environment variables.
///
/// This is designed to be used as a part of a Cargo build script and sets the following
/// environment variables:
///
/// * `FURIOSA_GIT_SHORT_HASH`
/// * `FURIOSA_BUILD_TIMESTAMP`
pub fn set_metadata_env_vars() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(VarError::NotPresent) = env::var("FURIOSA_GIT_SHORT_HASH") {
        println!(
            "cargo:rustc-env=FURIOSA_GIT_SHORT_HASH={}",
            git_short_hash()?
        );
    }

    println!(
        "cargo:rustc-env=FURIOSA_BUILD_TIMESTAMP={}",
        build_timestamp()
    );

    Ok(())
}

/// Returns the Git short hash for the current branch of the npu-tools repository.
///
/// The hash will have a `-modified` suffix if the repository is dirty.
fn git_short_hash() -> Result<String, Box<dyn std::error::Error>> {
    const WORKSPACE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");

    let repository = Repository::open(WORKSPACE_DIR)?;
    let git_hash = repository.revparse_single("HEAD")?.id().to_string();
    let mut git_short_hash = git_hash[..9].to_string();

    let mut status_options = StatusOptions::new();
    status_options.include_ignored(false);
    status_options.include_untracked(false);
    status_options.exclude_submodules(true);
    let dirty = (repository.statuses(Some(&mut status_options))?.iter())
        .any(|status| !(matches!(status.status(), Status::CURRENT)));
    if dirty {
        git_short_hash.push_str("-modified");
    }

    Ok(git_short_hash)
}

/// Returns the date and time of the current build.
fn build_timestamp() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    set_metadata_env_vars()?;
    Ok(())
}
