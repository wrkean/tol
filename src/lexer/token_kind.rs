#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Paraan,
    Ang,
    Maiba,
    Ibalik,
    Bagay,
    Itupad,
    Kung,
    KungDi,
    KungWala,

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
    LeftBracket,
    RightBracket,
    Colon,
    ColonColon,
    Comma,
    Dot,
    SemiColon,
    ThinArrow,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    At,
    Question,
    Bang,

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
