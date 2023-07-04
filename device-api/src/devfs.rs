use std::fs::FileType;
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use regex::{Match, Regex};

use crate::{DeviceError, DeviceResult};

lazy_static! {
    // Update MATCH_PATTERN_NUM when you change this pattern
    static ref DEVICE_FILE_PATTERN: Regex =
    Regex::new(r"^npu(?P<device_id>\d+)((?:pe)(?P<start_core>\d+)(-(?P<end_core>\d+))?)?$").unwrap();
}

pub(crate) fn path<P: AsRef<Path>>(base_path: P, filename: &str) -> PathBuf {
    base_path.as_ref().join(filename)
}

pub(crate) fn is_character_device(file_type: FileType) -> bool {
    // allow just a file too for unit testing
    if cfg!(test) {
        file_type.is_file()
    } else {
        file_type.is_char_device()
    }
}

pub(crate) fn parse_indices<S: AsRef<str>>(filename: S) -> DeviceResult<(u8, Vec<u8>)> {
    let name = filename.as_ref();
    let matches = DEVICE_FILE_PATTERN
        .captures(name)
        .ok_or_else(|| DeviceError::unrecognized_file(name))?;

    let device_id = parse_id(name, matches.name("device_id"));
    let core_start = parse_id(name, matches.name("start_core"));
    let end_core = parse_id(name, matches.name("end_core"));

    let (device_id, core_ids) = match (device_id, core_start, end_core) {
        (Some(device_id), None, None) => (device_id?, vec![]),
        (Some(device_id), Some(start_core), None) => (device_id?, vec![start_core?]),
        (Some(device_id), Some(start_core), Some(end_core)) => {
            (device_id?, (start_core?..=end_core?).collect())
        }
        _ => return Err(DeviceError::unrecognized_file(name)),
    };

    Ok((device_id, core_ids))
}

fn parse_id(name: &str, m: Option<Match<'_>>) -> Option<DeviceResult<u8>> {
    m.map(|i| {
        i.as_str()
            .parse()
            .map_err(|_| DeviceError::unrecognized_file(name))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_pattern() {
        let items = DEVICE_FILE_PATTERN.captures("npu0").unwrap();

        assert_eq!("npu0", items.get(0).unwrap().as_str());
        assert_eq!("0", items.name("device_id").unwrap().as_str());
        assert!(items.name("start_core").is_none());

        // Only start_core
        let items = DEVICE_FILE_PATTERN.captures("npu0pe4").unwrap();
        assert_eq!("npu0pe4", items.get(0).unwrap().as_str());
        assert_eq!("0", items.name("device_id").unwrap().as_str());
        assert_eq!("4", items.name("start_core").unwrap().as_str());
        assert!(items.name("end_core").is_none());

        // Only start_core - end_core
        let items = DEVICE_FILE_PATTERN.captures("npu0pe4-7").unwrap();
        assert_eq!("npu0pe4-7", items.get(0).unwrap().as_str());
        assert_eq!("0", items.name("device_id").unwrap().as_str());
        assert_eq!("4", items.name("start_core").unwrap().as_str());
        assert_eq!("7", items.name("end_core").unwrap().as_str());

        // incomplete case
        assert!(DEVICE_FILE_PATTERN.captures("npu0pe").is_none());
        assert!(DEVICE_FILE_PATTERN.captures("npu0pe0-").is_none());
        assert!(DEVICE_FILE_PATTERN.captures("npu0pe-9").is_none());
    }

    #[test]
    fn test_parse_indices() -> DeviceResult<()> {
        assert_eq!(parse_indices("npu0")?, (0, vec![]));
        assert_eq!(parse_indices("npu3pe4")?, (3, vec![4]));
        assert_eq!(parse_indices("npu3pe4-7")?, (3, vec![4, 5, 6, 7]));

        // incomplete cases
        assert!(parse_indices("npu").is_err());
        assert!(parse_indices("npu0pe").is_err());
        assert!(parse_indices("npu0pe0-").is_err());

        Ok(())
    }
}
