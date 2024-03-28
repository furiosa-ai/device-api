use pyo3::prelude::*;

/// Enum for the NPU architecture.
#[pyclass(name = "Arch")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ArchPy {
    WarboyA0,
    // It indicates WarboyB0 since WarboyB0 is the default
    Warboy,
    Renegade,
    U250,
}

#[pyclass(name = "ArchFamily")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ArchFamilyPy {
    Warboy,
    Renegade,
}

impl From<ArchFamilyPy> for furiosa_device::ArchFamily {
    fn from(arch_family: ArchFamilyPy) -> Self {
        match arch_family {
            ArchFamilyPy::Warboy => furiosa_device::ArchFamily::Warboy,
            ArchFamilyPy::Renegade => furiosa_device::ArchFamily::Renegade,
        }
    }
}
