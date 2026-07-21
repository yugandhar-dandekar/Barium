use crate::token::{self, Token};
use std::path::Path;

// allow printing
#[derive(Debug)]
pub enum Error {
    FailedToAdvance,
    FailedToIndexSource,
    UnexpectedEOF,
    ErroneousEscapeCharacter,
    UnterminatedCharLiteral,
}

pub struct Lexer {
    // source will be a list of u8 characters
    pub source: Vec<u8>,
    tokens: Vec<Token>,

    // used for string slices
    start: usize,
    current: usize,

    // to find what line a token is on
    line: usize,
}

impl Lexer {
    /// Default constructor for `Lexer`
    #[allow(dead_code)]
    pub fn new(source: Vec<u8>) -> Self {
        let source_len = source.len();

        Self {
            source,
            tokens: Vec::with_capacity(source_len / 3),

            // this will point to the start of each token, the length is 'current' - 'start'
            start: 0,

            current: 0, // this will always point to the next character being lexed
            line: 1,
        }
    }

    /// Constructor from file
    #[allow(dead_code)]
    pub fn from_file(path: &Path) -> Result<Self, std::io::Error> {
        let source = std::fs::read(path)?;
        Ok(Self::new(source))
    }

    /// Checks if `index` passed is at the end of the file.
    ///
    /// The lexer works by pointing to the character that will be processed next. Hence, 'index'
    /// will have reached the end when it points to the location after the last character. For
    /// example, the word 'test' has 4 characters and the last 't' has an index of 3,
    /// 'index_is_at_end' will return true if a value >= 4 is passed
    ///
    /// # Example
    ///
    /// ```
    /// // self.source = "test"
    ///
    /// assert_eq!(self.index_is_at_end(4), true);
    /// assert_eq!(self.index_is_at_end(3), false); // index of 't'
    /// ```
    fn index_is_at_end(&self, index: usize) -> bool {
        index >= self.source.len()
    }

    /// Checks if `self.current` is at the end
    fn is_at_end(&self) -> bool {
        self.index_is_at_end(self.current)
    }

    /// Returns the character at `index` if exists
    #[must_use]
    fn peek_index(&self, index: usize) -> Option<u8> {
        self.source.get(index).copied() // &u8 to u8
    }

    /// Returns the character currently being processed
    #[must_use]
    fn peek(&self) -> Option<u8> {
        self.peek_index(self.current)
    }

    /// Returns the character after the current character being processed
    #[must_use]
    fn peek_next(&self) -> Option<u8> {
        self.peek_index(self.current + 1)
    }

    /// Increases `self.current` by `value`
    fn advance_by(&mut self, value: usize) -> Result<(), Error> {
        let new_index = self.current + value;

        // allow advancing to the end but not past the end
        if self.index_is_at_end(new_index) && new_index != self.source.len() {
            Err(Error::FailedToAdvance)
        } else {
            self.current = new_index;
            Ok(())
        }
    }

    /// Increments `self.current`
    fn advance(&mut self) -> Result<(), Error> {
        self.advance_by(1)
    }

    /// Peeks the current character and advances if possible
    fn peek_and_advance(&mut self) -> Result<u8, Error> {
        let character = self.peek().ok_or({
            // if not ok

            if self.is_at_end() {
                Error::UnexpectedEOF // if the code has already reached the end
            } else {
                Error::FailedToIndexSource // if the code fails to peek
            }
        })?; // if not ok break and return error

        self.advance()?; // return the error if failed to advance

        Ok(character) // return the character
    }

    /// Adds a token to `self.tokens` and decides the `line` and `lexeme` attribute automatically
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

    /// Advances if the current character being processed matches 'expected'
    fn advance_if_match(&mut self, expected: u8) -> Result<bool, Error> {
        if self.peek() == Some(expected) {
            self.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Processes the current character and decides how to handle the token
    fn scan(&mut self) -> Result<(), Error> {
        // get the current character
        let character = self.peek_and_advance()?;

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
                if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::EqEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Equal)
                }
            }

            b'<' => {
                if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::LessEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Less)
                }
            }

            b'>' => {
                if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::GreaterEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Greater)
                }
            }

            b'!' => {
                if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::NotEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Not)
                }
            }

            b'&' => {
                if self.advance_if_match(b'&')? {
                    self.add_token_automatically(token::TokenTypes::And)
                } else {
                    self.add_token_automatically(token::TokenTypes::Ampersand)
                }
            }

            b'|' => {
                if self.advance_if_match(b'|')? {
                    self.add_token_automatically(token::TokenTypes::Or)
                } else {
                    self.add_token_automatically(token::TokenTypes::Pipe)
                }
            }

            // operators
            b'+' => {
                if self.advance_if_match(b'+')? {
                    self.add_token_automatically(token::TokenTypes::PlusPlus)
                } else if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::PlusEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Plus)
                }
            }

            b'-' => {
                if self.advance_if_match(b'-')? {
                    self.add_token_automatically(token::TokenTypes::MinusMinus)
                } else if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::MinusEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Minus)
                }
            }

            b'*' => {
                if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::AsteriskEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Asterisk)
                }
            }

            b'/' => {
                if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::SlashEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Slash)
                }
            }

            b'%' => {
                if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::PercentEqual)
                } else {
                    self.add_token_automatically(token::TokenTypes::Percent)
                }
            }

            b'\n' => {
                self.line += 1;
                self.add_token_automatically(token::TokenTypes::EndOfLine)
            }

            // files on windows end each line with \r\n, on other systems it may end with
            //   \n, to treat this the same the \r can just be ignored
            b'\r' => {} // do nothing for carriage return

            b'\t' => {
                while self.peek() == Some(b'\t') {
                    self.advance()?;
                }

                // self.add_token_automatically(token::TokenTypes::Whitespace)?
            }

            b' ' => {
                while self.peek() == Some(b' ') {
                    self.advance()?;
                }

                // self.add_token_automatically(token::TokenTypes::Whitespace)?
            }

            b'"' => self.handle_string_literal()?,
            b'\'' => self.handle_character_literal()?,

            // default case
            _ => {
                // identifiers are start with either 'a-z/A-Z' or '_'
                if character.is_ascii_alphabetic() || character == b'_' {
                    while self
                        .peek()
                        // if c exists, check if it is alphanumeric or '_'
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == b'_')
                    {
                        self.advance()?;
                    }
                    self.add_token_automatically(token::TokenTypes::Identifier);
                // handle numbers
                } else if character.is_ascii_digit() {
                    self.handle_number_literal()?;
                // default case add a token of type unknown
                } else {
                    self.add_token_automatically(token::TokenTypes::Unknown);
                }
            }
        }

        Ok(())
    }

    /// Handles string literal token type and automatically adds it to `self.tokens`
    fn handle_string_literal(&mut self) -> Result<(), Error> {
        // while '"' or end is not reached advance
        while self.peek() != Some(b'"') && !self.is_at_end() {
            // if there is a newline increment the line count
            if self.peek() == Some(b'\n') {
                self.line += 1;
            }

            self.advance()?;
        }

        // if advancing has ended because of 'is_at_end()' being true, return error early
        if self.is_at_end() {
            return Err(Error::UnexpectedEOF);
        }

        // go past the ending double quote as it has been processed
        self.advance()?;

        // if 'get_source_string' is not None, add the token with appropriate lexeme
        self.add_token_automatically(token::TokenTypes::CharString);

        Ok(())
    }

    /// Handles number tokens and automatically adds it to `self.tokens`
    fn handle_number_literal(&mut self) -> Result<(), Error> {
        // check whether 'peek' is Some character. If it exists, check if it is an ascii digit
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.advance()?;
        }

        if !self.is_at_end()
            && self.peek() == Some(b'.')
            // if 'peek_next' is None, map it to false
            // else check if it is an ascii digit
            && self.peek_next().map_or(false, |c| c.is_ascii_digit())
        {
            // skip the dot
            self.advance()?;

            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.advance()?;
            }

            self.add_token_automatically(token::TokenTypes::Float);
        } else {
            self.add_token_automatically(token::TokenTypes::Integer);
        }

        Ok(())
    }

    fn handle_character_literal(&mut self) -> Result<(), Error> {
        if self.peek() == Some(b'\\') {
            self.advance()?; // go past the '\'

            let escape_char = self.peek_and_advance()?;

            match escape_char {
                b'n' | b't' | b'r' | b'\\' | b'\'' | b'"' | b'0' => {}
                _ => return Err(Error::ErroneousEscapeCharacter),
            }
        } else {
            // regular single character: consume it
            self.peek_and_advance()?;
        }

        if self.peek() != Some(b'\'') {
            return Err(Error::UnterminatedCharLiteral);
        }
        self.advance()?; // go past end '

        self.add_token_automatically(token::TokenTypes::Char);

        Ok(())
    }

    /// Public method to lex the entire source provided and returns it as a list of tokens
    pub fn lex_text(&mut self) -> Result<Vec<Token>, Error> {
        // repeat until the code is at the end
        while !self.is_at_end() {
            // set the start of the next token to the current index
            self.start = self.current;
            self.scan()?;
        }

        // add end token
        self.add_token_manually(
            token::TokenTypes::EndOfFile,
            self.current,
            self.current,
            self.line,
        );

        // takes ownership of tokens and returns it
        Ok(std::mem::take(&mut self.tokens))
    }
}
