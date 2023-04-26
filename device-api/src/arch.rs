use std::fmt::{Display, Formatter};

use strum_macros::AsRefStr;

/// Enum for the NPU architecture.
#[derive(AsRefStr, Clone, Copy, Debug, enum_utils::FromStr, Eq, PartialEq)]
#[enumeration(case_insensitive)]
pub enum Arch {
    WarboyA0,
    #[enumeration(alias = "Warboy")]
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
            WarboyA0 => write!(f, "warboy-a0"),
            WarboyB0 => write!(f, "warboy"),
            Renegade => write!(f, "renegade"),
            U250 => write!(f, "u250"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_archkind() {
        assert!(Arch::from_str("Warboy").is_ok());
    }
}
