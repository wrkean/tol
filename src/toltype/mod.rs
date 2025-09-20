use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TolType {
    // Integers
    // Signed
    I8,
    I16,
    I32,
    I64,
    ISukat,

    // Unsigned
    U8,
    U16,
    U32,
    U64,
    USukat,

    // Floating-points
    Lutang,
    DobleTang,

    // Unsized
    // Supposedly, these types are not visible
    // to users, so they are in english
    UnsizedInt,
    UnsizedFloat,

    // Others
    Bool,
    Kar,
    Wala,

    Unknown,
}

impl TolType {
    fn is_integer(&self) -> bool {
        matches!(
            self,
            TolType::I8
                | TolType::I16
                | TolType::I32
                | TolType::I64
                | TolType::ISukat
                | TolType::U8
                | TolType::U16
                | TolType::U32
                | TolType::U64
                | TolType::USukat
                | TolType::UnsizedInt
        )
    }

    fn is_float(&self) -> bool {
        matches!(
            self,
            TolType::Lutang | TolType::DobleTang | TolType::UnsizedFloat
        )
    }

    pub fn is_arithmetic_compatible(&self, other: &Self) -> bool {
        (self.is_integer() && other.is_integer()) || (self.is_float() && other.is_float())
    }

    pub fn is_assignment_compatible(&self, other: &Self) -> bool {
        (self.is_integer() && (other.is_integer() || other == &TolType::UnsizedInt))
            || (self.is_float() && (other.is_float() || other == &TolType::UnsizedFloat))
            || (self == other)
    }
}

impl fmt::Display for TolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TolType::I8 => write!(f, "i8"),
            TolType::I16 => write!(f, "i16"),
            TolType::I32 => write!(f, "i32"),
            TolType::I64 => write!(f, "i64"),
            TolType::ISukat => write!(f, "isukat"),
            TolType::U8 => write!(f, "u8"),
            TolType::U16 => write!(f, "u16"),
            TolType::U32 => write!(f, "u32"),
            TolType::U64 => write!(f, "u64"),
            TolType::USukat => write!(f, "usukat"),
            TolType::Lutang => write!(f, "lutang"),
            TolType::DobleTang => write!(f, "dobletang"),
            TolType::Bool => write!(f, "bool"),
            TolType::Kar => write!(f, "kar"),
            TolType::Wala => write!(f, "wala"),
            _ => write!(f, "<hindi_tipo>"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_arithmetic_compatibility() {
        assert!(TolType::I8.is_arithmetic_compatible(&TolType::I64));
        assert!(TolType::U8.is_arithmetic_compatible(&TolType::U64));
        assert!(!TolType::I32.is_arithmetic_compatible(&TolType::Lutang));
        assert!(!TolType::I64.is_arithmetic_compatible(&TolType::DobleTang));
    }
}
