use pyo3::prelude::*;

use crate::arch::ArchPy;
use crate::device::{CoreRangePy, DeviceModePy};

#[pyclass(name = "DeviceConfigInner")]
pub struct DeviceConfigInnerPy {}

#[pyclass(name = "Named")]
pub struct NamedPy {
    device_id: u8,
    core_range: CoreRangePy,
}

#[pyclass(name = "Unnamed")]
pub struct UnnamedPy {
    arch: ArchPy,
    core_num: u8,
    mode: DeviceModePy,
    count: u8,
}

#[pyclass(name = "Config")]
pub struct ConfigPy {
    Named: Option<NamedPy>,
    Unnamed: Option<UnnamedPy>,
}

// enum ConfigPyEnum {
//     Named(NamedPy),
//     Unnamed(UnnamedPy)
// }

// #[pyclass]
// pub struct ConfigPy {
//     inner: ConfigPyEnum
// }
