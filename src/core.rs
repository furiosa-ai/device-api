use itertools::join;
use lazy_static::lazy_static;
use regex::{Regex, Captures};
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::{self, Display};
use std::path::PathBuf;

use crate::device::DeviceType;

#[derive(Debug, Eq, PartialEq)]
pub struct Core {
    idx: u8,
    path: PathBuf,
    core_type: CoreType,
    device_type: DeviceType,
    status: CoreStatus,
}

impl Core {
    pub (crate) fn new(idx: u8, path: PathBuf, core_type: CoreType, device_type: DeviceType, status: CoreStatus) -> Self {
        Self {
            idx,
            path,
            core_type,
            device_type,
            status,
        }
    }

    pub (crate) fn with_status(self, status: CoreStatus) -> Self {
        Self {
            status,
            ..self
        }
    }

    pub fn name(&self) -> String {
        format!("npu{}{}", self.idx, self.core_type)
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn core_type(&self) -> &CoreType {
        &self.core_type
    }

    pub fn device_idx(&self) -> u8 {
        self.idx
    }

    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }

    pub fn status(&self) -> CoreStatus {
        self.status
    }

    pub fn core_count(&self) -> u8 {
        self.core_type.count()
    }

    pub fn is_fusioned(&self) -> bool {
        self.core_count() > 1
    }
}

impl Ord for Core {
    fn cmp(&self, other: &Self) -> Ordering {
        self.idx.cmp(&other.idx)
            .then(self.core_type.cmp(&other.core_type))
    }
}

impl PartialOrd for Core {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Display, Copy, Clone, Eq, PartialEq)]
pub enum CoreStatus {
    Available,
    Occupied,
    Occupied2,
    Unavailable
}

type CoreIdx = u8;
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CoreType {
    Single(CoreIdx),
    Fusion(Vec<CoreIdx>),
}

impl Display for CoreType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            CoreType::Single(pe) => format!(":{}", pe),
            CoreType::Fusion(v) => format!(":{}", join(v, "-")),
        };

        write!(f, "{}", name)
    }
}

impl CoreType {
    fn count(&self) -> u8 {
        match self {
            CoreType::Single(_) => 1,
            CoreType::Fusion(v) => u8::try_from(v.len()).unwrap()
        }
    }
}

lazy_static! {
    static ref REGEX_PE: Regex = Regex::new(r"^(npu)(?P<npu>\d*)(pe)(?P<pe>\d+)$").unwrap();
    static ref REGEX_FUSION: Regex = Regex::new(r"^(npu)(?P<npu>\d*)(pe)(?P<pe>(\d+-)+\d+)$").unwrap();
}

fn capture_to_str<'a>(c: &'a Captures, key: &'a str) -> &'a str {
  c.name(key).unwrap().as_str()
}

impl TryFrom<&str> for CoreType {
    type Error = ();

    fn try_from(item: &str) -> Result<Self, Self::Error> {
        if let Some(x) = REGEX_PE.captures(item) {
            Ok(CoreType::Single(
              capture_to_str(&x, "pe").parse().unwrap()
            ))
        } else if let Some(x) = REGEX_FUSION.captures(item) {
            let indexes: Vec<u8> = capture_to_str(&x, "pe").split("-")
                .map(|s| s.parse().unwrap())
                .collect();

            Ok(CoreType::Fusion(
                indexes
            ))
        } else {
            Err(())
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from() {
        assert_eq!(CoreType::try_from("npu0"), Err(()));
        assert_eq!(CoreType::try_from("npu0pe"), Err(()));
        assert_eq!(CoreType::try_from("npu0pe0"), Ok(CoreType::Single(0)));
        assert_eq!(CoreType::try_from("npu0pe1"), Ok(CoreType::Single(1)));
        assert_eq!(CoreType::try_from("npu0pe0-1"), Ok(CoreType::Fusion(vec![0, 1])));
        assert_eq!(CoreType::try_from("npu0pe0-1-2"), Ok(CoreType::Fusion(vec![0, 1, 2])));
        assert_eq!(CoreType::try_from("npu0pe0-"), Err(()));
        assert_eq!(CoreType::try_from("npu0pe-1"), Err(()));
    }

    #[test]
    fn test_fmt() {
        assert_eq!(format!("{}", CoreType::Single(0)), ":0");
        assert_eq!(format!("{}", CoreType::Single(1)), ":1");

        assert_eq!(format!("{}", CoreType::Fusion(vec![0, 1])), ":0-1");
        assert_eq!(format!("{}", CoreType::Fusion(vec![0, 1, 2, 3])), ":0-1-2-3");
    }
}
