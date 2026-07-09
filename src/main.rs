use std::path::Path;

use crate::token::Token;

mod lexer;
mod token;

fn print_tokens(tokens: Vec<Token>) {
    for token in tokens {
        match token.token_type {
            token::TokenTypes::Whitespace | token::TokenTypes::EndOfFile => {}
            _ => {
                println!("{:?}", token);
            }
        }
    }
}

fn main() {
    let mut lexer = lexer::Lexer::from_file(Path::new("./.ignore/test")).unwrap();

    let tokens = lexer.lex_text();

    match tokens {
        Ok(tokens) => print_tokens(tokens),
        Err(err) => {
            eprintln!("{:?}", err)
        }
    }
}
