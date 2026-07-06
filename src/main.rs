use std::fs;

mod lexer;
mod token;

fn main() {
    let contents: String = fs::read_to_string(".ignore/test").expect("Failed to read file");
    let mut lexer = lexer::Lexer::new(&contents.as_bytes());

    let tokens = lexer.lex_text().unwrap();

    for token in tokens {
        match token.token_type {
            // token::TokenTypes::Whitespace | token::TokenTypes::EndOfFile => {}
            _ => {
                println!("{:?}", token);
            }
        }
    }
}
