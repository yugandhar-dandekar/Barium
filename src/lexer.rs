#![allow(dead_code, unused_imports)]

use crate::token::{self, Token};

#[derive(Debug)]
pub enum Error {
    FailedToPeek,
    FailedToAdvance,
    FailedToIndexSource,
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

            start: 0,
            current: 0,
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

    pub fn get_source_slice(&self, start: usize, end: usize) -> Result<&[u8], Error> {
        // start must not be greater than end
        // end is not inclusive so it can be equal to 'source_len'
        if start > end || end > self.source_len {
            return Err(Error::FailedToIndexSource);
        }

        Ok(&self.source[start..end])
    }
}
