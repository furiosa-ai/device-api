use pyo3::prelude::*;

/// Enum for the NPU architecture.
#[pyclass(name = "Arch")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ArchPy {
    Warboy,
    #[allow(clippy::upper_case_acronyms)]
    RNGD,
}

impl From<ArchPy> for furiosa_device::Arch {
    fn from(arch_family: ArchPy) -> Self {
        match arch_family {
            ArchPy::Warboy => furiosa_device::Arch::WarboyB0,
            ArchPy::RNGD => furiosa_device::Arch::RNGD,
        }
    }
}
