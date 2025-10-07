use core::panic;
use std::fmt;

pub mod type_info;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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
    Sinulid,

    // Composite
    Bagay(String),
    UnknownIdentifier(String),

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
        if self == &TolType::UnsizedInt {
            return other.is_integer();
        }

        if self == &TolType::UnsizedFloat {
            return other.is_float();
        }

        self == other
    }

    pub fn as_c(&self) -> String {
        match self {
            TolType::I8 => "int8_t",
            TolType::I16 => "int16_t",
            TolType::I32 => "int32_t",
            TolType::I64 => "int64_t",
            TolType::ISukat => "ptrdiff_t",
            TolType::U8 => "uint8_t",
            TolType::U16 => "uint16_t",
            TolType::U32 => "uint32_t",
            TolType::U64 => "uint64_t",
            TolType::USukat => "size_t",
            TolType::Lutang => "float",
            TolType::DobleTang => "double",
            TolType::Bool => "bool",
            TolType::Kar => "char",
            TolType::Wala => "void",
            TolType::Sinulid => "char*",
            _ => {
                // Semantic analyzer already checks if the types are valid, so this maybe won't
                // trigger
                panic!("If this panic! gets triggered, something is VERY wrong with the semantic analyzer")
            }
        }
        .to_string()
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
            TolType::Sinulid => write!(f, "sinulid"),
            TolType::UnsizedInt => write!(f, "literal na integer"),
            TolType::UnsizedFloat => write!(f, "literal na lutang"),
            _ => write!(f, "<hindi_tipo>"),
        }
    }
}
//
// #[cfg(test)]
// mod test {
//     use super::*;
//
//     #[test]
//     fn test_arithmetic_compatibility() {
//         assert!(TolType::I8.is_arithmetic_compatible(&TolType::I64));
//         assert!(TolType::U8.is_arithmetic_compatible(&TolType::U64));
//         assert!(!TolType::I32.is_arithmetic_compatible(&TolType::Lutang));
//         assert!(!TolType::I64.is_arithmetic_compatible(&TolType::DobleTang));
//     }
// }
