use pyo3::prelude::*;

#[pyclass(name = "Arch")]
pub enum ArchPy {
    Warboy,
    WarboyB0,
    Renegade,
    U250,
}
