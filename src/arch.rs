use std::fmt::{Display, Formatter};
use strum_macros::{AsRefStr, EnumString};

#[derive(AsRefStr, Clone, Copy, Debug, EnumString, Eq, PartialEq)]
pub enum Arch {
    Warboy,
    WarboyB0,
    Renegade,
    U250, /* TODO - It's somewhat ambiguous. We need two attributes to distinguish both HW type
           * and NPU family. */
}

impl Display for Arch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Arch::*;

        // Keep the same as npu-id of Compiler to display
        match self {
            Warboy => write!(f, "warboy"),
            WarboyB0 => write!(f, "warboy-b0"),
            Renegade => write!(f, "renegade"),
            U250 => write!(f, "u250"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::arch::Arch;
    use std::str::FromStr;

    #[test]
    fn test_archkind() {
        assert!(Arch::from_str("Warboy").is_ok());
    }
}
