use colored::Colorize;
use std::env;
use std::time::Instant;

mod lexer;
mod token;

fn main() {
    let args: Vec<String> = env::args().collect();
    let source: Vec<u8> = std::fs::read_to_string(&args[1]).unwrap().into_bytes();

    let mut lexer = lexer::Lexer::new(source);

    let start = Instant::now();
    let tokens = match lexer.lex_text() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("{}", format!("Error while lexing: {e}").red());
            std::process::exit(1);
        }
    };
    let elapsed = start.elapsed();

    for token in tokens {
        println!("{:?}", token.token_type);
    }

    println!("Elapsed: {:?}", elapsed);
}
