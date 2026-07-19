#![allow(dead_code)]

// #[derive(Debug, PartialEq)]
pub enum TokenTypes {
    // special token types
    Unknown, // type is none of the others
    EndOfLine,
    EndOfFile,
    Whitespace, // tab or 4 spaces

    // literal token types
    Identifier, // for all words
    Keyword,    // if word lexed matches one of the keywords
    CharString, // contained within double quotes, e.g. "Test"
    Integer,    // a series of connected numbers, e.g. 73847
    Float,      // 2 series of connected numbers joined by one full stop, e.g. 384.384

    // single character token types
    // bracketed token types
    LParenthesis, // (
    RParenthesis, // )
    LBracket,     // [
    RBracket,     // ]
    LBrace,       // {
    RBrace,       // }

    // single character operator/special types
    Plus,     // +
    Minus,    // -
    Asterisk, // *
    Slash,    // /
    Percent,  // %

    // double character operator
    PlusEqual,     // +=
    MinusEqual,    // -=
    AsteriskEqual, // *=
    SlashEqual,    // /=
    PercentEqual,  // %=

    PlusPlus,   // ++ increment
    MinusMinus, // -- decrement

    Comma,     // ,
    Dot,       // .
    Semicolon, // ;
    Colon,     // :
    Ampersand, // &
    Pipe,      // |
    Caret,     // ^
    Tilde,     // ~
    Equal,     // =

    // single comparison token types
    Greater, // >
    Less,    // <

    // double comparison token types
    EqEqual,      // ==
    GreaterEqual, // >=
    LessEqual,    // <=
    NotEqual,     // !=

    // logic token types
    And, // &&
    Or,  // ||
    Not, // !
}

pub struct Token {
    pub token_type: TokenTypes,
    pub start: usize,
    pub end: usize,
    pub line: usize,
}
