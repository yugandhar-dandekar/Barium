use std::path::Path;

mod lexer;
mod token;

fn main() {
    let mut lexer = lexer::Lexer::from_file(Path::new("./.ignore/test")).unwrap();

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
