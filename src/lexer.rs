use crate::token::{self, Token};

#[derive(Debug)]
pub enum Error {
    FailedToPeek,
    FailedToAdvance,
    FailedToIndexSource,
    UnexpectedEOF,
}

pub struct Lexer<'a> {
    // source will be a list of u8 characters
    pub source: &'a [u8],
    pub tokens: Vec<Token>,

    // fixed length of source
    pub source_len: usize,

    // used for string slices
    pub start: usize,
    pub current: usize,

    // to find what line a token is on
    pub line: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self {
            source: source,
            tokens: Vec::new(),

            source_len: source.len(),

            start: 0, // this will point to the start of each token, the length is 'current' - 'start'
            current: 0, // this will always point to the next character being lexed
            line: 1,
        }
    }

    pub fn index_is_at_end(&self, index: usize) -> bool {
        index >= self.source_len
    }

    pub fn is_at_end(&self) -> bool {
        self.index_is_at_end(self.current)
    }

    pub fn peek_index(&self, index: usize) -> Result<u8, Error> {
        if !self.index_is_at_end(index) {
            Ok(self.source[index])
        } else {
            Err(Error::FailedToPeek)
        }
    }

    pub fn peek(&self) -> Result<u8, Error> {
        self.peek_index(self.current)
    }

    pub fn peek_next(&self) -> Result<u8, Error> {
        self.peek_index(self.current + 1)
    }

    pub fn advance_by(&mut self, value: usize) -> Result<(), Error> {
        let new_index = self.current + value;

        // allow advancing to the end
        if self.index_is_at_end(new_index) && new_index != self.source_len {
            return Err(Error::FailedToAdvance);
        }

        self.current = new_index;

        Ok(())
    }

    pub fn advance(&mut self) -> Result<(), Error> {
        self.advance_by(1)
    }

    pub fn peek_and_advance(&mut self) -> Result<u8, Error> {
        let character = self.peek()?; // return the error if failed to peek

        self.advance()?; // return the error if failed to advance

        Ok(character) // return the character
    }

    /// start is inclusive, end is not inclusive
    pub fn get_source_slice(&self, start: usize, end: usize) -> Result<&[u8], Error> {
        // start must not be greater than end
        // end is not inclusive so it can be equal to 'source_len'
        if start > end || end > self.source_len {
            Err(Error::FailedToIndexSource)
        } else {
            Ok(&self.source[start..end])
        }
    }

    /// add token with parameters set manually
    pub fn add_token_manually(
        &mut self,
        token_type: token::TokenTypes,
        lexeme: Box<str>,
        line: usize,
    ) {
        let token = Token {
            token_type,
            lexeme,
            line,
        };

        self.tokens.push(token);
    }

    /// add token with parameters decided automatically
    pub fn add_token_automatically(&mut self, token_type: token::TokenTypes) -> Result<(), Error> {
        let line = self.line;

        let lexeme = self
            .reference_array_to_box_str(self.get_source_slice(self.start, self.current)?)
            .expect("Failed to convert lexeme into Box<str>");

        let token = Token {
            token_type,
            lexeme,
            line,
        };

        self.tokens.push(token);

        Ok(())
    }

    /// only advance if the next character == expected
    pub fn advance_if_match(&mut self, expected: u8) -> Result<bool, Error> {
        if self.peek()? == expected {
            self.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// converts &[u8] to Box<str>
    pub fn reference_array_to_box_str(&self, u8_array: &[u8]) -> Result<Box<str>, ()> {
        let box_str: Box<str> = str::from_utf8(u8_array).map_err(|_| ())?.into();

        Ok(box_str)
    }

    /// scans a token, if any action fails, it will return with an error that can be
    ///   handled separately
    pub fn scan(&mut self) -> Result<(), Error> {
        // get the current character
        let character = self.peek_and_advance()?;

        match character {
            b')' => self.add_token_automatically(token::TokenTypes::RParenthesis)?,
            b'(' => self.add_token_automatically(token::TokenTypes::LParenthesis)?,

            b']' => self.add_token_automatically(token::TokenTypes::RBracket)?,
            b'[' => self.add_token_automatically(token::TokenTypes::LBracket)?,

            b'{' => self.add_token_automatically(token::TokenTypes::LBrace)?,
            b'}' => self.add_token_automatically(token::TokenTypes::RBrace)?,

            b',' => self.add_token_automatically(token::TokenTypes::Comma)?,
            b'.' => self.add_token_automatically(token::TokenTypes::Dot)?,
            b';' => self.add_token_automatically(token::TokenTypes::Semicolon)?,
            b':' => self.add_token_automatically(token::TokenTypes::Colon)?,
            b'^' => self.add_token_automatically(token::TokenTypes::Caret)?,
            b'~' => self.add_token_automatically(token::TokenTypes::Tilde)?,

            b'=' => {
                if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::EqEqual)?
                } else {
                    self.add_token_automatically(token::TokenTypes::Equal)?
                }
            }

            b'<' => {
                if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::LessEqual)?
                } else {
                    self.add_token_automatically(token::TokenTypes::Less)?
                }
            }

            b'>' => {
                if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::GreaterEqual)?
                } else {
                    self.add_token_automatically(token::TokenTypes::Greater)?
                }
            }

            b'!' => {
                if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::NotEqual)?
                } else {
                    self.add_token_automatically(token::TokenTypes::Not)?
                }
            }

            b'&' => {
                if self.advance_if_match(b'&')? {
                    self.add_token_automatically(token::TokenTypes::And)?
                } else {
                    self.add_token_automatically(token::TokenTypes::Ampersand)?
                }
            }

            b'|' => {
                if self.advance_if_match(b'|')? {
                    self.add_token_automatically(token::TokenTypes::Or)?
                } else {
                    self.add_token_automatically(token::TokenTypes::Pipe)?
                }
            }

            // operators
            b'+' => {
                if self.advance_if_match(b'+')? {
                    self.add_token_automatically(token::TokenTypes::PlusPlus)?
                } else if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::PlusEqual)?
                } else {
                    self.add_token_automatically(token::TokenTypes::Plus)?
                }
            }

            b'-' => {
                if self.advance_if_match(b'-')? {
                    self.add_token_automatically(token::TokenTypes::MinusMinus)?
                } else if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::MinusEqual)?
                } else {
                    self.add_token_automatically(token::TokenTypes::Minus)?
                }
            }

            b'*' => {
                if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::AsteriskEqual)?
                } else {
                    self.add_token_automatically(token::TokenTypes::Asterisk)?
                }
            }

            b'/' => {
                if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::SlashEqual)?
                } else {
                    self.add_token_automatically(token::TokenTypes::Slash)?
                }
            }

            b'%' => {
                if self.advance_if_match(b'=')? {
                    self.add_token_automatically(token::TokenTypes::PercentEqual)?
                } else {
                    self.add_token_automatically(token::TokenTypes::Percent)?
                }
            }

            b'\n' => {
                self.line += 1;
                self.add_token_automatically(token::TokenTypes::EndOfLine)?
            }

            // files on windows end each line with \r\n, on other systems it may end with
            //   \n, to treat this the same the \r can just be ignored
            b'\r' => {} // do nothing for carriage return

            b'\t' => {
                while !self.is_at_end() && self.peek()? == b'\t' {
                    self.advance()?;
                }

                self.add_token_automatically(token::TokenTypes::Whitespace)?
            }

            b' ' => {
                while !self.is_at_end() && self.peek()? == b' ' {
                    self.advance()?;
                }

                self.add_token_automatically(token::TokenTypes::Whitespace)?
            }

            b'"' => self.handle_string_literal()?,
            _ => {
                if character.is_ascii_alphabetic() || character == b'_' {
                    while !self.is_at_end()
                        && (self.peek()?.is_ascii_alphanumeric() || self.peek()? == b'_')
                    {
                        self.advance()?;
                    }

                    self.add_token_automatically(token::TokenTypes::Identifier)?
                } else if character.is_ascii_digit() {
                    self.handle_number_literal()?;
                } else {
                    self.add_token_automatically(token::TokenTypes::Unknown)?
                }
            }
        }

        Ok(())
    }

    pub fn handle_string_literal(&mut self) -> Result<(), Error> {
        while !self.is_at_end() && self.peek()? != b'"' {
            self.advance()?;
        }

        // if the advance has ended because of an EOF throw an error
        if self.is_at_end() {
            return Err(Error::UnexpectedEOF);
        }

        self.advance()?;

        // only get the characters in between the double quotes
        let lexeme = self
            .reference_array_to_box_str(self.get_source_slice(self.start + 1, self.current - 1)?)
            .expect("Failed to convert lexeme into Box<str>");

        self.add_token_manually(token::TokenTypes::CharString, lexeme, self.line);

        Ok(())
    }

    pub fn handle_number_literal(&mut self) -> Result<(), Error> {
        while !self.is_at_end() && self.peek()?.is_ascii_digit() {
            self.advance()?;
        }

        if !self.is_at_end()
            && self.peek()? == b'.'
            && self.peek_next().map_or(false, |c| c.is_ascii_digit())
        {
            // skip the dot
            self.advance()?;

            while !self.is_at_end() && self.peek()?.is_ascii_digit() {
                self.advance()?;
            }

            self.add_token_automatically(token::TokenTypes::Float)?;
        } else {
            self.add_token_automatically(token::TokenTypes::Integer)?;
        }

        Ok(())
    }

    pub fn lex_text(&mut self) -> Result<Vec<Token>, Error> {
        // repeat until the code is at the end
        while !self.is_at_end() {
            // set the start of the next token to the current index
            self.start = self.current;
            self.scan()?;
        }

        // add end token
        self.add_token_manually(token::TokenTypes::EndOfFile, Box::from("EOF"), self.line);

        // takes ownership of tokens and returns it
        Ok(std::mem::take(&mut self.tokens))
    }
}
