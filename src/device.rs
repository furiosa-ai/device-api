use std::convert::TryFrom;
use std::fmt::Display;

#[derive(Debug, Display, Copy, Clone, Eq, PartialEq)]
pub enum DeviceType {
    Warboy,
    WarboyB0,
    Renegade,
}

impl TryFrom<&str> for DeviceType {
    type Error = ();

    fn try_from(item: &str) -> Result<Self, Self::Error> {
        match item.trim() {
            "Warboy" => Ok(DeviceType::Warboy),
            _ => Err(())
        }
    }
}
