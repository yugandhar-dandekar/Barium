use std::env;
use std::time::Instant;

mod lexer;
mod token;

fn main() {
    let args: Vec<String> = env::args().collect();
    let source: Vec<u8> = std::fs::read_to_string(&args[1]).unwrap().into_bytes();

    let mut lexer = lexer::Lexer::new(source);

    let start = Instant::now();
    let tokens = lexer.lex_text().unwrap();
    let elapsed = start.elapsed();

    for token in tokens {
        println!("{:?}", token.token_type);
    }

    println!("Elapsed: {:?}", elapsed);
}
