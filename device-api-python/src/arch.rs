use pyo3::prelude::*;

#[pyclass(name = "Arch")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ArchPy {
    Warboy,
    WarboyB0,
    Renegade,
    U250,
}
