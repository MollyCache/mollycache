#[derive(Debug, PartialEq)]
pub enum TokenTypes {
    // Keywords:
    Create,
    Select,
    Insert,
    Table,
    From,
    Into,
    Values,
    Where,
    // Single-char tokens:
    Asterix,
    SemiColon,
    LeftParen,
    RightParen,
    Comma,
    Equals,
    // Literals:
    String,
    Number,
    // Identifier
    Identifier,

    // Other
    Error,
}
