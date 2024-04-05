use pyo3::prelude::*;

/// Enum for the NPU architecture.
#[pyclass(name = "Arch")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ArchPy {
    // It indicates WarboyB0 since WarboyB0 is the default
    Warboy,
    Renegade,
}

impl From<ArchPy> for furiosa_device::Arch {
    fn from(arch_family: ArchPy) -> Self {
        match arch_family {
            ArchPy::Warboy => furiosa_device::Arch::WarboyB0,
            ArchPy::Renegade => furiosa_device::Arch::Renegade,
        }
    }
}
