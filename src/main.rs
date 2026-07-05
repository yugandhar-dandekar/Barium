use std::str;

mod lexer;
mod token;

fn main() {
    let lexer = lexer::Lexer::new(b"Test");

    let characters = lexer.get_source_slice(2, 4);

    println!("{}", str::from_utf8(characters.unwrap()).unwrap())
}
