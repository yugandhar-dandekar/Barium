use crate::token::{self, Token};

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
    UnexpectedEOF { line: usize, col: usize },

    UnterminatedStringLiteral { line: usize, col: usize },

    UnterminatedCharLiteral { line: usize, col: usize },
    BadCharLiteral { line: usize, col: usize },
    EmptyCharLiteral { line: usize, col: usize },
    BadEscapeCharLiteral { line: usize, col: usize },
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

pub type LexerResult<T> = core::result::Result<T, LexerError>;
pub type TokenList = Vec<Token>;

pub struct Lexer<'a> {
    // assume source is UTF-8 encoded
    source: &'a [u8],
    source_len: usize,

    tokens: TokenList,

    // pointers to the start and end of each token
    start: usize,
    current: usize,

    line: usize,
}

impl<'a> Lexer<'a> {
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
    pub fn new(source: &'a Vec<u8>) -> Self {
        let source_len = source.len();

        Self {
            source,
            source_len,
            // assume less than 4 bytes per token average so estimated capacity
            // is len / 3 at maximum
            tokens: Vec::with_capacity(source_len / 2),

            start: 0,

            current: 0,
            line: 1,
        }
    }

    #[inline(always)]
    fn index_is_at_end(&self, index: usize) -> bool {
        index >= self.source_len
    }

    #[inline(always)]
    fn is_at_end(&self) -> bool {
        self.index_is_at_end(self.current)
    }

    #[must_use]
    #[inline(always)]
    fn peek_index(&self, index: usize) -> Option<u8> {
        self.source.get(index).copied()
    }

    #[must_use]
    #[inline(always)]
    fn peek(&self) -> Option<u8> {
        self.peek_index(self.current)
    }

    /// Unsafe consumption
    #[inline(always)]
    fn consume_unchecked(&mut self) {
        self.current += 1;
    }

    #[inline(always)]
    fn newline(&mut self) {
        self.line += 1;
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
        self.tokens.push(Token {
            token_type,
            start: start as u32,
            end: end as u32,
            line: line as u32,
        });
    }

    #[inline(always)]
    fn consume_if_match(&mut self, expected: u8) -> bool {
        if self.peek().is_some_and(|c| c == expected) {
            self.consume_unchecked();
            true
        } else {
            false
        }
    }

    fn scan(&mut self) -> LexerResult<()> {
        // peek the current character, since its already
        // processed, advance automatically
        let character =
            self.peek()
                .ok_or(LexerError::Internal(InternalError::FailedToIndexSource {
                    line: self.line,
                    current: self.current,
                }))?;

        self.consume_unchecked();

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
                if self.consume_if_match(b'=') {
                    self.add_token_automatically(token::TokenTypes::EqEqual)
                } else if self.consume_if_match(b'>') {
                    self.add_token_automatically(token::TokenTypes::RArrowThick)
                } else {
                    self.add_token_automatically(token::TokenTypes::Equal)
                }
            }

            b'<' => {
                if self.consume_if_match(b'=') {
                    self.add_token_automatically(token::TokenTypes::LessEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Less)
                }
            }

            b'>' => {
                if self.consume_if_match(b'=') {
                    self.add_token_automatically(token::TokenTypes::GreaterEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Greater)
                }
            }

            b'!' => {
                if self.consume_if_match(b'=') {
                    self.add_token_automatically(token::TokenTypes::NotEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Not)
                }
            }

            b'&' => {
                if self.consume_if_match(b'&') {
                    self.add_token_automatically(token::TokenTypes::And)
                } else {
                    self.add_token_automatically(token::TokenTypes::Ampersand)
                }
            }

            b'|' => {
                if self.consume_if_match(b'|') {
                    self.add_token_automatically(token::TokenTypes::Or)
                } else {
                    self.add_token_automatically(token::TokenTypes::Pipe)
                }
            }

            b'+' => {
                if self.consume_if_match(b'+') {
                    self.add_token_automatically(token::TokenTypes::PlusPlus)
                } else if self.consume_if_match(b'=') {
                    self.add_token_automatically(token::TokenTypes::PlusEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Plus)
                }
            }

            b'-' => {
                if self.consume_if_match(b'-') {
                    self.add_token_automatically(token::TokenTypes::MinusMinus)
                } else if self.consume_if_match(b'=') {
                    self.add_token_automatically(token::TokenTypes::MinusEqual)
                } else if self.consume_if_match(b'>') {
                    self.add_token_automatically(token::TokenTypes::RArrowThin)
                } else {
                    self.add_token_automatically(token::TokenTypes::Minus)
                }
            }

            b'*' => {
                if self.consume_if_match(b'=') {
                    self.add_token_automatically(token::TokenTypes::AsteriskEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Asterisk)
                }
            }

            b'/' => {
                if self.consume_if_match(b'=') {
                    self.add_token_automatically(token::TokenTypes::SlashEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Slash)
                }
            }

            b'%' => {
                if self.consume_if_match(b'=') {
                    self.add_token_automatically(token::TokenTypes::PercentEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Percent)
                }
            }

            b'\n' => {
                self.newline();
                self.add_token_automatically(token::TokenTypes::EndOfLine)
            }

            b'\r' => {}

            b' ' | b'\t' => {
                while matches!(self.peek_index(self.current), Some(b' ') | Some(b'\t')) {
                    self.current += 1;
                }
            }

            b'"' => self.handle_string_literal()?,
            b'\'' => self.handle_char_literal()?,

            _ => {
                if character.is_ascii_alphabetic() || character == b'_' {
                    self.handle_identifier();
                } else if character.is_ascii_digit() {
                    self.handle_number_literal();
                } else {
                    self.add_token_automatically(token::TokenTypes::Unknown);
                }
            }
        }

        Ok(())
    }

    fn handle_identifier(&mut self) {
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == b'_')
        {
            self.consume_unchecked();
        }

        self.add_token_automatically(token::TokenTypes::Identifier);
    }

    fn handle_string_literal(&mut self) -> LexerResult<()> {
        while let Some(c) = self.peek() {
            if c == b'"' {
                break;
            }

            if c == b'\n' {
                self.newline();
            }

            self.consume_unchecked();
        }

        if self.is_at_end() {
            return Err(LexerError::Source(SourceError::UnterminatedStringLiteral {
                line: self.line,
                col: self.start + 1,
            }));
        }

        self.consume_unchecked(); // consume the closing quote

        self.add_token_manually(
            token::TokenTypes::CharString,
            self.start + 1,
            self.current - 1,
            self.line,
        );

        Ok(())
    }

    fn handle_number_literal(&mut self) {
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.consume_unchecked();
        }

        if self.peek().is_some_and(|c| c == b'.') {
            self.consume_unchecked();

            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.consume_unchecked();
            }

            self.add_token_automatically(token::TokenTypes::Float);
        } else {
            self.add_token_automatically(token::TokenTypes::Integer);
        }
    }

    fn handle_char_literal(&mut self) -> LexerResult<()> {
        let char = self
            .peek()
            .ok_or(LexerError::Source(SourceError::EmptyCharLiteral {
                line: self.line,
                col: self.start + 1,
            }))?;

        // prevent ''
        if char == b'\'' {
            return Err(LexerError::Source(SourceError::EmptyCharLiteral {
                line: self.line,
                col: self.start + 1,
            }));
        } else if char == b'\\' {
            self.consume_unchecked();

            let escape_char =
                self.peek()
                    .ok_or(LexerError::Source(SourceError::UnterminatedCharLiteral {
                        line: self.line,
                        col: self.start + 1,
                    }))?;

            match escape_char {
                b'n' | b't' | b'r' | b'\\' | b'\'' | b'"' | b'0' => {}
                _ => {
                    return Err(LexerError::Source(SourceError::BadEscapeCharLiteral {
                        line: self.line,
                        col: self.current + 1,
                    }));
                }
            }
        }

        self.consume_unchecked();

        let terminating_char =
            self.peek()
                .ok_or(LexerError::Source(SourceError::UnterminatedCharLiteral {
                    line: self.line,
                    col: self.start + 1,
                }))?;

        if terminating_char != b'\'' {
            return Err(LexerError::Source(SourceError::UnterminatedCharLiteral {
                line: self.line,
                col: self.current + 1,
            }));
        }

        self.consume_unchecked();

        self.add_token_manually(
            token::TokenTypes::Char,
            self.start + 1,
            self.current - 1,
            self.line,
        );

        Ok(())
    }

    pub fn lex_text(&mut self) -> LexerResult<()> {
        while !self.is_at_end() {
            // set start to the current so the token start is recorded
            self.start = self.current;

            match self.scan() {
                Ok(()) => {}
                Err(err) => return Err(err),
            }
        }

        // add the end of file token
        self.add_token_manually(
            token::TokenTypes::EndOfFile,
            self.current,
            self.current,
            self.line,
        );

        Ok(())
    }

    pub fn take_token(&mut self) -> TokenList {
        // take ownership of tokens
        std::mem::take(&mut self.tokens)
    }
}
