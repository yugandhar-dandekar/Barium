use crate::token::{self, Token};
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum Error {
    FailedToAdvance,
    FailedToIndexSource,
    UnexpectedEOF,
}

pub struct Lexer {
    // source will be a list of u8 characters
    pub source: Vec<u8>,
    pub tokens: Vec<Token>,

    // used for string slices
    pub start: usize,
    pub current: usize,

    // to find what line a token is on
    pub line: usize,
}

impl Lexer {
    #[allow(dead_code)]
    pub fn new(source: Vec<u8>) -> Self {
        Self {
            source: source,
            tokens: Vec::new(),

            start: 0, // this will point to the start of each token, the length is 'current' - 'start'
            current: 0, // this will always point to the next character being lexed
            line: 1,
        }
    }

    #[allow(dead_code)]
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        // if unsuccessful, break flow and return the error
        Ok(Self::new(fs::read(path)?))
    }

    pub fn index_is_at_end(&self, index: usize) -> bool {
        // index is at the end when the current character being processed is
        //   past the end of the characters, for example, if we had the word
        //   'test' the 4th character won't be at the end because the character
        //   still hasn't been processed
        index >= self.source.len()
    }

    pub fn is_at_end(&self) -> bool {
        self.index_is_at_end(self.current)
    }

    pub fn peek_index(&self, index: usize) -> Option<u8> {
        self.source.get(index).copied() // &u8 to u8
    }

    pub fn peek(&self) -> Option<u8> {
        self.peek_index(self.current)
    }

    pub fn peek_next(&self) -> Option<u8> {
        self.peek_index(self.current + 1)
    }

    pub fn advance_by(&mut self, value: usize) -> Result<(), Error> {
        let new_index = self.current + value;

        // allow advancing to the end
        if self.index_is_at_end(new_index) && new_index != self.source.len() {
            Err(Error::FailedToAdvance)
        } else {
            self.current = new_index;
            Ok(())
        }
    }

    pub fn advance(&mut self) -> Result<(), Error> {
        self.advance_by(1)
    }

    pub fn peek_and_advance(&mut self) -> Result<u8, Error> {
        let character = self.peek().ok_or({
            if self.is_at_end() {
                Error::UnexpectedEOF // if the code has already reached the end
            } else {
                Error::FailedToIndexSource // if the code fails to peek
            }
        })?; // return the error if failed to peek

        self.advance()?; // return the error if failed to advance

        Ok(character) // return the character
    }

    fn get_source_string(&self, start: usize, end: usize) -> Option<Box<str>> {
        let slice = self
            .source
            .get(start..end)
            .map_or(None, |s| std::str::from_utf8(s).ok())?; // break if from_utf8 fails

        Some(slice.into()) // convert slice into Box<str>
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
            .get_source_string(self.start, self.current)
            .ok_or(Error::FailedToIndexSource)?;

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
        if self.peek() == Some(expected) {
            self.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
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
                while self.peek() == Some(b'\t') {
                    self.advance()?;
                }

                self.add_token_automatically(token::TokenTypes::Whitespace)?
            }

            b' ' => {
                while self.peek() == Some(b' ') {
                    self.advance()?;
                }

                self.add_token_automatically(token::TokenTypes::Whitespace)?
            }

            b'"' => self.handle_string_literal()?,
            _ => {
                if character.is_ascii_alphabetic() || character == b'_' {
                    while self
                        .peek()
                        // checks if c exists and c is a alphanumeric or _
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == b'_')
                    {
                        self.advance()?;
                    }
                    self.add_token_automatically(token::TokenTypes::Identifier)?;
                } else if character.is_ascii_digit() {
                    self.handle_number_literal()?;
                } else {
                    self.add_token_automatically(token::TokenTypes::Unknown)?;
                }
            }
        }

        Ok(())
    }

    pub fn handle_string_literal(&mut self) -> Result<(), Error> {
        while self.peek() != Some(b'"') && !self.is_at_end() {
            if self.peek() == Some(b'\n') {
                self.line += 1;
            }

            self.advance()?;
        }

        // if the advance has ended because of an EOF throw an error
        if self.is_at_end() {
            return Err(Error::UnexpectedEOF);
        }

        // go past the ending double quote as it has been processed
        self.advance()?;

        // only get the characters in between the double quotes
        let lexeme = self
            .get_source_string(self.start + 1, self.current - 1)
            .ok_or(Error::FailedToIndexSource)?;

        self.add_token_manually(token::TokenTypes::CharString, lexeme, self.line);

        Ok(())
    }

    pub fn handle_number_literal(&mut self) -> Result<(), Error> {
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.advance()?;
        }

        if !self.is_at_end()
            && self.peek() == Some(b'.')
            && self.peek_next().map_or(false, |c| c.is_ascii_digit())
        {
            // skip the dot
            self.advance()?;

            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
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
