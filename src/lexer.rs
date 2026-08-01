use crate::token::{self, Token};
use std::path::Path;

#[derive(Debug)]
pub enum LexerError {
    Internal(InternalError),
    Source(SourceError),
}

#[derive(Debug)]
pub enum InternalError {
    FailedToAdvance { line: usize, current: usize },
    FailedToIndexSource { line: usize, current: usize },
}

#[derive(Debug)]
pub enum SourceError {
    UnexpectedEOF { line: usize, current: usize },

    UnterminatedStringLiteral { line: usize, current: usize },
    UnterminatedCharLiteral { line: usize, current: usize },
    BadEscapeCharacter { line: usize, current: usize },
}

impl From<InternalError> for LexerError {
    fn from(err: InternalError) -> Self {
        LexerError::Internal(err)
    }
}

impl From<SourceError> for LexerError {
    fn from(err: SourceError) -> Self {
        LexerError::Source(err)
    }
}

pub type LResult<T> = core::result::Result<T, LexerError>;

pub struct Lexer {
    // assume source is UTF-8 encoded
    source: Vec<u8>,

    tokens: Vec<Token>,

    // pointers to the start and end of each token
    start: usize,
    current: usize,

    line: usize,
}

impl Lexer {
    /// Default constructor for `Lexer`
    ///
    /// # Example
    ///
    /// ```
    /// use barium::Lexer;
    ///
    /// let mut lexer = Lexer::new(b"hello".into());
    /// let tokens = lexer.lex_text();
    /// ```
    #[allow(dead_code)]
    pub fn new(source: Vec<u8>) -> Self {
        let source_len = source.len();

        Self {
            source,
            // assume less than 4 bytes per token average so estimated capacity
            // is len / 3 at maximum
            tokens: Vec::with_capacity(source_len / 3),

            start: 0,

            current: 0,
            line: 1,
        }
    }

    /// Constructs Lexer from `path`
    ///
    /// Attempts to read `path`. If failed, the method will return [`std::io::Error`]
    /// early
    #[allow(dead_code)]
    pub fn from_file(path: &Path) -> Result<Self, std::io::Error> {
        let source = std::fs::read(path)?;

        Ok(Self::new(source))
    }

    /// Checks if `index` provided is at the end of the file
    ///
    /// `index` will have reached the end when `index` == the length of the
    /// source, this is because the `current` attribute in [`Lexer`] always
    /// points to the next character being processed which is when it is 1
    /// more than the index of the last character.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// self.source = b"test".into();
    ///
    /// // equal to the length of "test" so reached the end
    /// assert_eq!(self.index_is_at_end(4), true);
    ///
    /// // greater than the length of "test" so gone past the end
    /// assert_eq!(self.index_is_at_end(10), true);
    ///
    /// // not reached the end
    /// assert_eq!(self.index_is_at_end(3), false);
    /// ```
    fn index_is_at_end(&self, index: usize) -> bool {
        index >= self.source.len()
    }

    /// Checks if `current` attribute in Lexer is at the end
    ///
    /// Wrapper method for Lexer::index_is_at_end
    fn is_at_end(&self) -> bool {
        self.index_is_at_end(self.current)
    }

    /// Peeks `source` at `index`
    ///
    /// returns [`Option<u8>`]
    #[must_use]
    fn peek_index(&self, index: usize) -> Option<u8> {
        self.source.get(index).copied()
    }

    /// Peeks `source` at `current` index
    ///
    /// returns [`Option<u8>`]
    #[must_use]
    fn peek(&self) -> Option<u8> {
        self.peek_index(self.current)
    }

    /// Peeks `source` at `current + 1` index
    ///
    /// returns [`Option<u8>`]
    #[must_use]
    fn peek_next(&self) -> Option<u8> {
        self.peek_index(self.current + 1)
    }

    /// Consumes `n` characters in `source`
    ///
    /// increments `current` index by `n`
    fn consume_n(&mut self, n: usize) -> LResult<()> {
        let new_index = self.current + n;

        // only allow consuming `n` if the index isn't at the end or exactly
        // equal to the length of source so that all characters can be
        // consumed
        if !self.index_is_at_end(new_index) || new_index == self.source.len() {
            self.current = new_index;
            Ok(())
        } else {
            Err(LexerError::Internal(InternalError::FailedToAdvance {
                line: self.line,
                current: self.current,
            }))
        }
    }

    /// Consumes 1 character in `source`
    fn consume(&mut self) -> LResult<()> {
        self.consume_n(1)
    }

    fn peek_and_consume(&mut self) -> LResult<u8> {
        // peek the current character. If the peek fails, it could either be
        // because `current` has reached the end of `source` or because of a
        // failure to peek, handle that error here
        let character = self.peek().ok_or(InternalError::FailedToIndexSource {
            line: (self.line),
            current: (self.current),
        })?;

        // consume the peeked character
        self.consume()?;

        Ok(character)
    }

    /// Pushes a token to `tokens` with parameters decided automatically
    ///
    /// sets `end` is set to `current`, all other parameters are the same
    fn add_token_automatically(&mut self, token_type: token::TokenTypes) {
        self.add_token_manually(token_type, self.start, self.current, self.line);
    }

    /// Pushes a token to `tokens`
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

    /// If the peeked character is equal to the expected, return true and advance,
    /// else return false. If the peek fails, return an error
    fn consume_if_match(&mut self, expected: u8) -> LResult<bool> {
        if self.peek() == Some(expected) {
            self.consume()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn scan(&mut self) -> Result<(), LexerError> {
        // peek the current character, since its already
        // processed, advance automatically
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

    fn handle_string_literal(&mut self) -> Result<(), LexerError> {
        while self.peek() != Some(b'"') && !self.is_at_end() {
            if self.peek() == Some(b'\n') {
                self.line += 1;
            }

            self.consume()?;
        }

        if self.is_at_end() {
            Err(LexerError::Source(SourceError::UnexpectedEOF {
                line: self.line,
                current: self.current,
            }))?
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

    fn handle_number_literal(&mut self) -> LResult<()> {
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

    fn handle_character_literal(&mut self) -> LResult<()> {
        if self.peek() == Some(b'\\') {
            self.consume()?;

            let escape_char = self.peek_and_consume()?;

            match escape_char {
                b'n' | b't' | b'r' | b'\\' | b'\'' | b'"' | b'0' => {}
                _ => {
                    return Err(LexerError::Source(SourceError::BadEscapeCharacter {
                        line: self.line,
                        current: self.current,
                    }));
                }
            }
        } else {
            self.peek_and_consume()?;
        }

        if self.peek() != Some(b'\'') {
            return Err(LexerError::Source(SourceError::UnterminatedCharLiteral {
                line: self.line,
                current: self.current,
            }));
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

    pub fn lex_text(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            // set start to the current so the token start is recorded
            self.start = self.current;

            match self.scan() {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("Lexing error: {:?}", err);
                }
            }
        }

        // add the end of file token
        self.add_token_manually(
            token::TokenTypes::EndOfFile,
            self.current,
            self.current,
            self.line,
        );

        // take ownership of tokens
        std::mem::take(&mut self.tokens)
    }
}
