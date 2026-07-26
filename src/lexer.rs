use crate::token::{self, Token};
use std::fmt;
use std::path::Path;

pub enum Error {
    FailedToAdvance { line: usize, current: usize },
    FailedToIndexSource { line: usize, current: usize },

    UnexpectedEOF { line: usize },
    ErroneousEscapeCharacter { line: usize, found: u8 },
    UnterminatedCharLiteral { line: usize },
}

// for printing of Error
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::FailedToAdvance { line, current } => {
                write!(
                    f,
                    "line {line}: failed to advance lexer past position {current}"
                )
            }
            Error::FailedToIndexSource { line, current } => {
                write!(
                    f,
                    "line {line}: failed to read source at position {current}"
                )
            }
            Error::UnexpectedEOF { line } => {
                write!(f, "line {line}: unexpected end of file")
            }
            Error::ErroneousEscapeCharacter { line, found } => {
                write!(
                    f,
                    "line {line}: invalid escape character '\\{}' in character literal",
                    *found as char
                )
            }
            Error::UnterminatedCharLiteral { line } => {
                write!(f, "line {line}: character literal is missing a closing '")
            }
        }
    }
}

pub struct Lexer {
    source: Vec<u8>,
    tokens: Vec<Token>,

    start: usize,
    current: usize,

    line: usize,
}

impl Lexer {
    #[allow(dead_code)]
    pub fn new(source: Vec<u8>) -> Self {
        let source_len = source.len();

        Self {
            source,
            tokens: Vec::with_capacity(source_len / 3),

            start: 0,

            current: 0,
            line: 1,
        }
    }

    #[allow(dead_code)]
    pub fn from_file(path: &Path) -> Result<Self, std::io::Error> {
        let source = std::fs::read(path)?;
        Ok(Self::new(source))
    }

    fn index_is_at_end(&self, index: usize) -> bool {
        index >= self.source.len()
    }

    fn is_at_end(&self) -> bool {
        self.index_is_at_end(self.current)
    }

    #[must_use]
    fn peek_index(&self, index: usize) -> Option<u8> {
        self.source.get(index).copied()
    }

    #[must_use]
    fn peek(&self) -> Option<u8> {
        self.peek_index(self.current)
    }

    #[must_use]
    fn peek_next(&self) -> Option<u8> {
        self.peek_index(self.current + 1)
    }

    fn consume_n(&mut self, n: usize) -> Result<(), Error> {
        let new_index = self.current + n;

        if self.index_is_at_end(new_index) && new_index != self.source.len() {
            Err(Error::FailedToAdvance {
                line: self.line,
                current: self.current,
            })
        } else {
            self.current = new_index;
            Ok(())
        }
    }

    fn consume(&mut self) -> Result<(), Error> {
        self.consume_n(1)
    }

    fn peek_and_consume(&mut self) -> Result<u8, Error> {
        let character = self.peek().ok_or({
            if self.is_at_end() {
                Error::UnexpectedEOF { line: self.line }
            } else {
                Error::FailedToIndexSource {
                    line: self.line,
                    current: self.current,
                }
            }
        })?;

        self.consume()?;

        Ok(character)
    }

    fn add_token_automatically(&mut self, token_type: token::TokenTypes) {
        self.add_token_manually(token_type, self.start, self.current, self.line);
    }

    fn add_token_manually(
        &mut self,
        token_type: token::TokenTypes,
        start: usize,
        end: usize,
        line: usize,
    ) {
        let token = Token {
            token_type,
            start: start as u32,
            end: end as u32,
            line: line as u32,
        };

        self.tokens.push(token);
    }
    fn consume_if_match(&mut self, expected: u8) -> Result<bool, Error> {
        if self.peek() == Some(expected) {
            self.consume()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn scan(&mut self) -> Result<(), Error> {
        let character = self.peek_and_consume()?;

        match character {
            b')' => self.add_token_automatically(token::TokenTypes::RParenthesis),
            b'(' => self.add_token_automatically(token::TokenTypes::LParenthesis),

            b']' => self.add_token_automatically(token::TokenTypes::RBracket),
            b'[' => self.add_token_automatically(token::TokenTypes::LBracket),

            b'{' => self.add_token_automatically(token::TokenTypes::LBrace),
            b'}' => self.add_token_automatically(token::TokenTypes::RBrace),

            b',' => self.add_token_automatically(token::TokenTypes::Comma),
            b'.' => self.add_token_automatically(token::TokenTypes::Dot),
            b';' => self.add_token_automatically(token::TokenTypes::Semicolon),
            b':' => self.add_token_automatically(token::TokenTypes::Colon),
            b'^' => self.add_token_automatically(token::TokenTypes::Caret),
            b'~' => self.add_token_automatically(token::TokenTypes::Tilde),

            b'=' => {
                if self.consume_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::EqEqual)
                } else if self.consume_if_match(b'>')? {
                    self.add_token_automatically(token::TokenTypes::RArrowThick)
                } else {
                    self.add_token_automatically(token::TokenTypes::Equal)
                }
            }

            b'<' => {
                if self.consume_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::LessEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Less)
                }
            }

            b'>' => {
                if self.consume_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::GreaterEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Greater)
                }
            }

            b'!' => {
                if self.consume_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::NotEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Not)
                }
            }

            b'&' => {
                if self.consume_if_match(b'&')? {
                    self.add_token_automatically(token::TokenTypes::And)
                } else {
                    self.add_token_automatically(token::TokenTypes::Ampersand)
                }
            }

            b'|' => {
                if self.consume_if_match(b'|')? {
                    self.add_token_automatically(token::TokenTypes::Or)
                } else {
                    self.add_token_automatically(token::TokenTypes::Pipe)
                }
            }

            b'+' => {
                if self.consume_if_match(b'+')? {
                    self.add_token_automatically(token::TokenTypes::PlusPlus)
                } else if self.consume_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::PlusEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Plus)
                }
            }

            b'-' => {
                if self.consume_if_match(b'-')? {
                    self.add_token_automatically(token::TokenTypes::MinusMinus)
                } else if self.consume_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::MinusEqual)
                } else if self.consume_if_match(b'>')? {
                    self.add_token_automatically(token::TokenTypes::RArrowThin)
                } else {
                    self.add_token_automatically(token::TokenTypes::Minus)
                }
            }

            b'*' => {
                if self.consume_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::AsteriskEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Asterisk)
                }
            }

            b'/' => {
                if self.consume_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::SlashEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Slash)
                }
            }

            b'%' => {
                if self.consume_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::PercentEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Percent)
                }
            }

            b'\n' => {
                self.line += 1;
                self.add_token_automatically(token::TokenTypes::EndOfLine)
            }

            b'\r' => {}

            b'\t' => {
                while self.peek() == Some(b'\t') {
                    self.consume()?;
                }

                // self.add_token_automatically(token::TokenTypes::Whitespace)?
            }

            b' ' => {
                while self.peek() == Some(b' ') {
                    self.consume()?;
                }

                // self.add_token_automatically(token::TokenTypes::Whitespace)?
            }

            b'"' => self.handle_string_literal()?,
            b'\'' => self.handle_character_literal()?,

            _ => {
                if character.is_ascii_alphabetic() || character == b'_' {
                    while self
                        .peek()
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == b'_')
                    {
                        self.consume()?;
                    }
                    self.add_token_automatically(token::TokenTypes::Identifier);
                } else if character.is_ascii_digit() {
                    self.handle_number_literal()?;
                } else {
                    self.add_token_automatically(token::TokenTypes::Unknown);
                }
            }
        }

        Ok(())
    }

    fn handle_string_literal(&mut self) -> Result<(), Error> {
        while self.peek() != Some(b'"') && !self.is_at_end() {
            if self.peek() == Some(b'\n') {
                self.line += 1;
            }

            self.consume()?;
        }

        if self.is_at_end() {
            return Err(Error::UnexpectedEOF { line: self.line });
        }

        self.consume()?;

        self.add_token_manually(
            token::TokenTypes::CharString,
            self.start + 1,
            self.current - 1,
            self.line,
        );

        Ok(())
    }

    fn handle_number_literal(&mut self) -> Result<(), Error> {
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.consume()?;
        }

        if !self.is_at_end()
            && self.peek() == Some(b'.')
            && self.peek_next().map_or(false, |c| c.is_ascii_digit())
        {
            self.consume()?;

            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.consume()?;
            }

            self.add_token_automatically(token::TokenTypes::Float);
        } else {
            self.add_token_automatically(token::TokenTypes::Integer);
        }

        Ok(())
    }

    fn handle_character_literal(&mut self) -> Result<(), Error> {
        if self.peek() == Some(b'\\') {
            self.consume()?;

            let escape_char = self.peek_and_consume()?;

            match escape_char {
                b'n' | b't' | b'r' | b'\\' | b'\'' | b'"' | b'0' => {}
                _ => {
                    return Err(Error::ErroneousEscapeCharacter {
                        line: self.line,
                        found: escape_char,
                    });
                }
            }
        } else {
            self.peek_and_consume()?;
        }

        if self.peek() != Some(b'\'') {
            return Err(Error::UnterminatedCharLiteral { line: self.line });
        }
        self.consume()?;

        self.add_token_manually(
            token::TokenTypes::Char,
            self.start + 1,
            self.current - 1,
            self.line,
        );

        Ok(())
    }

    pub fn lex_text(&mut self) -> Result<Vec<Token>, Error> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan()?;
        }

        self.add_token_manually(
            token::TokenTypes::EndOfFile,
            self.current,
            self.current,
            self.line,
        );

        Ok(std::mem::take(&mut self.tokens))
    }
}
