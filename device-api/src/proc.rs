use std::fs;
use std::io::prelude::*;

use lazy_static::lazy_static;
use memoize::memoize;
use rayon::prelude::*;
use regex::Regex;

use crate::DeviceResult;

lazy_static! {
    static ref DEVICE_PATH_PATTERN: Regex = Regex::new(
        r"^/dev/npu(?P<device_id>\d+)((?:pe)(?P<start_core>\d+)(-(?P<end_core>\d+))?)?$"
    )
    .unwrap();
}

pub struct NpuProcess {
    pub dev_name: String,
    pub pid: u32,
    pub cmdline: String,
}

impl NpuProcess {
    pub(crate) fn new(dev_name: String, pid: u32, cmdline: String) -> Self {
        Self {
            dev_name,
            pid,
            cmdline,
        }
    }

    pub fn dev_name(&self) -> &str {
        self.dev_name.as_str()
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn cmdline(&self) -> &str {
        self.cmdline.as_str()
    }
}

pub fn scan_processes() -> DeviceResult<Vec<NpuProcess>> {
    let mut targets = Vec::new();

    for entry in fs::read_dir("/proc")? {
        if let Ok(pid) = entry?.file_name().to_string_lossy().parse::<u32>() {
            let path = format!("/proc/{pid}/fd");
            if let Ok(dirs) = fs::read_dir(&path) {
                for entry in dirs {
                    let entry = entry?;
                    targets.push((
                        pid,
                        format!(
                            "{}/{}",
                            path,
                            entry.file_name().as_os_str().to_string_lossy()
                        ),
                    ));
                }
            }
        }
    }

    let mut results: Vec<NpuProcess> = targets
        .into_par_iter()
        .filter_map(|(pid, path)| {
            if let Ok(link) = fs::read_link(path) {
                let link = link.as_os_str().to_string_lossy();
                if DEVICE_PATH_PATTERN.is_match(&link) {
                    return Some((pid, link.replace("/dev/", "")));
                }
            }
            None
        })
        .filter_map(|(pid, dev_path)| {
            read_cmdline(pid).map(|cmdline| NpuProcess::new(dev_path, pid, cmdline))
        })
        .collect();

    results.sort_by(|a, b| a.dev_name.cmp(&b.dev_name));

    Ok(results)
}

#[memoize(TimeToLive: std::time::Duration::from_secs(1))]
fn read_cmdline(pid: u32) -> Option<String> {
    let path = format!("/proc/{pid}/cmdline");
    let file = std::fs::File::open(path).ok()?;
    let mut buf_reader = std::io::BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).ok()?;

    Some(contents.replace('\0', " ").trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn read_cmdline_test() {
        let my_pid = std::process::id();
        let res = read_cmdline(my_pid);
        assert!(res.is_some());

        let res = res.unwrap();
        assert!(res.contains("/target/"));
        // when "cargo test": ".../device-api/target/debug/deps/furiosa_device-5a7434828b2c179b"
        // when module unit test: ".../device-api/target/debug/deps/furiosa_device-5a7434828b2c179b proc::tests --nocapture"
        // when function unit test: ".../device-api/target/debug/deps/furiosa_device-96a8035dd957f8e4 proc::tests::read_cmdline_test --exact --nocapture"
    }
}
