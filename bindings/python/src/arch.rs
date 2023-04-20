use pyo3::prelude::*;

#[pyclass(name = "Arch")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ArchPy {
    WarboyA0,
    // It indeicates WarboyB0 since WarboyB0 is default
    Warboy,
    Renegade,
    U250,
}
