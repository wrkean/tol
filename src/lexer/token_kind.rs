#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Paraan,
    Ang,
    Maiba,
    Ibalik,

    Identifier,

    // Literals,
    IntLit,
    FloatLit,
    StringLit,

    // Single-character literals
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Colon,
    Comma,
    SemiColon,
    ThinArrow,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    At,

    // Assignment operators
    Equal,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    PercentEqual,

    // Equality operators
    EqualEqual,
    BangEqual,

    // Relational operators
    Greater,
    GreaterEqual,
    Lesser,
    LesserEqual,

    Eof,
}
