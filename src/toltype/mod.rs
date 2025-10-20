use core::panic;
use std::fmt;

use crate::error::{CompilerError, ErrorKind};

pub mod type_info;

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
    // Str,

    // Composite
    Bagay(String),
    UnknownIdentifier(String),
    Array(Box<TolType>, Option<usize>),

    // Special
    AkoType,
    Unknown,
}

impl TolType {
    pub fn is_integer(&self) -> bool {
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

    pub fn is_assignment_compatible(
        &self,
        other: &Self,
        line: usize,
        column: usize,
    ) -> Result<(), CompilerError> {
        use TolType::*;

        // Helper to build errors
        let err = |msg: String| Err(CompilerError::new(&msg, ErrorKind::Error, line, column));

        if self == other {
            return Ok(());
        }

        match (self, other) {
            (UnsizedInt, o) | (UnsizedFloat, o) if o.is_integer() || o.is_float() => Ok(()),

            (Bagay(a), UnknownIdentifier(b)) | (UnknownIdentifier(a), Bagay(b)) if a == b => Ok(()),

            (Array(t1, right_len), Array(t2, left_len)) => {
                t1.is_assignment_compatible(t2, line, column)?;

                match (left_len, right_len) {
                    (Some(llen), Some(rlen)) if llen < rlen => err(format!(
                        "Mas maliit ang sukat na ibinigay sa tipo ({llen}) kumpara sa expresyon ({rlen})"
                    )),
                    (None, Some(0)) => err(
                        "Hindi pwede ang walang laman na array kung walang ibinigay na sukat sa tipo"
                            .into(),
                    ),
                    _ => Ok(()),
                }
            }

            // Fallback: incompatible types
            _ => err(format!(
                "Ang tipong `{}` ay hindi bagay sa tipong `{}`",
                self, other
            )),
        }
    }

    pub fn as_c(&self) -> String {
        match self {
            TolType::I8 => "int8_t".to_string(),
            TolType::I16 => "int16_t".to_string(),
            TolType::I32 => "int32_t".to_string(),
            TolType::I64 => "int64_t".to_string(),
            TolType::ISukat => "ptrdiff_t".to_string(),
            TolType::U8 => "uint8_t".to_string(),
            TolType::U16 => "uint16_t".to_string(),
            TolType::U32 => "uint32_t".to_string(),
            TolType::U64 => "uint64_t".to_string(),
            TolType::USukat => "size_t".to_string(),
            TolType::Lutang => "float".to_string(),
            TolType::DobleTang => "double".to_string(),
            TolType::Bool => "bool".to_string(),
            TolType::Kar => "char".to_string(),
            TolType::Wala => "void".to_string(),
            // TolType::Sinulid => "char*".to_string(),
            TolType::Bagay(s) => s.to_string(),
            TolType::UnknownIdentifier(s) => s.to_string(),
            TolType::Array(inner, _) => {
                let mut t = inner.as_ref();
                while let TolType::Array(next, _) = t {
                    t = next.as_ref();
                }
                t.as_c()
            }
            _ => {
                // Semantic analyzer already checks if the types are valid, so this maybe won't
                // trigger
                panic!(
                    "If this panic! gets triggered, something is VERY wrong with the semantic analyzer"
                )
            }
        }
    }

    // Special case for arrays because C array syntax
    // is weird
    pub fn array_suffix(&self) -> String {
        match self {
            TolType::Array(inner, len_opt) => {
                let inner_suffix = inner.array_suffix();
                match len_opt {
                    Some(len) => format!("[{}]{}", len, inner_suffix),
                    None => format!("[]{}", inner_suffix),
                }
            }
            _ => String::new(),
        }
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
            // TolType::Sinulid => write!(f, "sinulid"),
            TolType::UnsizedInt => write!(f, "literal na integer"),
            TolType::UnsizedFloat => write!(f, "literal na lutang"),
            TolType::Bagay(s) => write!(f, "{}", s),
            TolType::UnknownIdentifier(s) => write!(f, "{}", s),
            TolType::Array(t, _) => write!(f, "[{}]", t),
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
