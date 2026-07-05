#![allow(dead_code, unused_imports)]

use crate::token::{self, Token};

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
}
