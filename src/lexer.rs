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
            tokens: Vec::with_capacity(source_len / 3),

            start: 0,

            current: 0,
            line: 1,
        }
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
    #[inline(always)]
    fn index_is_at_end(&self, index: usize) -> bool {
        index >= self.source_len
    }

    /// Checks if `current` attribute in Lexer is at the end
    ///
    /// Wrapper method for Lexer::index_is_at_end
    #[inline(always)]
    fn is_at_end(&self) -> bool {
        self.index_is_at_end(self.current)
    }

    /// Peeks `source` at `index`
    ///
    /// returns [`Option<u8>`]
    #[must_use]
    #[inline(always)]
    fn peek_index(&self, index: usize) -> Option<u8> {
        self.source.get(index).copied()
    }

    /// Peeks `source` at `current` index
    ///
    /// returns [`Option<u8>`]
    #[must_use]
    fn peek(&self) -> LexerResult<u8> {
        self.peek_index(self.current).map_or_else(
            || {
                Err(LexerError::Internal(InternalError::FailedToIndexSource {
                    line: self.line,
                    current: self.current,
                }))
            },
            |c| Ok(c),
        )
    }

    /// Peeks `source` at `current + 1` index
    ///
    /// returns [`Option<u8>`]
    #[must_use]
    #[inline(always)]
    fn peek_next(&self) -> Option<u8> {
        self.peek_index(self.current + 1)
    }

    /// Unsafe consumption
    #[inline(always)]
    fn consume_unchecked(&mut self) {
        self.current += 1;
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
        self.tokens.push(Token {
            token_type,
            start: start as u32,
            end: end as u32,
            line: line as u32,
        });
    }

    /// If the peeked character is equal to the expected, return true and advance,
    /// else return false. If the peek fails, return an error
    #[inline(always)]
    fn consume_if_match(&mut self, expected: u8) -> bool {
        if self.peek().is_ok_and(|c| c == expected) {
            self.consume_unchecked();
            true
        } else {
            false
        }
    }

    fn scan(&mut self) -> LexerResult<()> {
        // peek the current character, since its already
        // processed, advance automatically
        let character = self.peek()?;

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
                self.line += 1;
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
                    self.add_token_automatically(token::TokenTypes::Identifier);
                } else if character.is_ascii_digit() {
                    self.handle_number_literal();
                } else {
                    self.add_token_automatically(token::TokenTypes::Unknown);
                }
            }
        }

        Ok(())
    }

    fn err_unterminated_string_literal(&self) -> LexerError {
        LexerError::Source(SourceError::UnterminatedStringLiteral {
            line: self.line,
            current: self.current,
        })
    }

    fn handle_identifier(&mut self) {
        while self
            .peek()
            .is_ok_and(|c| c.is_ascii_alphanumeric() || c == b'_')
        {
            self.consume_unchecked();
        }
    }

    fn handle_string_literal(&mut self) -> LexerResult<()> {
        while let Ok(c) = self.peek() {
            if c == b'"' {
                break;
            }

            if c == b'\n' {
                self.line += 1;
            }

            self.consume_unchecked();
        }

        if self.is_at_end() {
            return Err(self.err_unterminated_string_literal());
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
        while self.peek().is_ok_and(|c| c.is_ascii_digit()) {
            self.consume_unchecked();
        }

        if self.peek().is_ok_and(|c| c == b'.') {
            self.consume_unchecked();

            while self.peek().is_ok_and(|c| c.is_ascii_digit()) {
                self.consume_unchecked();
            }

            self.add_token_automatically(token::TokenTypes::Float);
        } else {
            self.add_token_automatically(token::TokenTypes::Integer);
        }
    }

    fn err_unterminated_char_literal(&self) -> LexerError {
        LexerError::Source(SourceError::UnterminatedCharLiteral {
            line: self.line,
            current: self.current,
        })
    }

    fn handle_char_literal(&mut self) -> LexerResult<()> {
        if self.peek().is_ok_and(|c| c == b'\\') {
            let escape_char =
                self.peek_next()
                    .ok_or(LexerError::Source(SourceError::BadEscapeCharacter {
                        line: self.line,
                        current: self.current,
                    }))?;

            self.consume_unchecked(); // consume the backslash

            match escape_char {
                b'n' | b't' | b'r' | b'\\' | b'\'' | b'"' | b'0' => {}
                _ => {
                    return Err(LexerError::Source(SourceError::BadEscapeCharacter {
                        line: self.line,
                        current: self.current,
                    }));
                }
            }

            self.consume_unchecked(); // consume the escape character itself
        } else {
            if self.is_at_end() {
                return Err(LexerError::Source(SourceError::BadEscapeCharacter {
                    line: self.line,
                    current: self.current,
                }));
            }

            self.consume_unchecked();
        }

        if !self.peek().is_ok_and(|c| c == b'\'') {
            return Err(self.err_unterminated_char_literal()); // or a new "char literal too long" variant
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

    pub fn lex_text(&mut self) {
        while !self.is_at_end() {
            // set start to the current so the token start is recorded
            self.start = self.current;

            match self.scan() {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("{:?}", err);
                    return;
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
    }

    pub fn take_token(&mut self) -> TokenList {
        // take ownership of tokens
        std::mem::take(&mut self.tokens)
    }
}
