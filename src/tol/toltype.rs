#[derive(Debug, PartialEq, Eq)]
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
