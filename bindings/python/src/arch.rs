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
