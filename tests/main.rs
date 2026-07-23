use colored::Colorize;
use std::env;
use std::time::Instant;

use barium_lexer::Lexer;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Expected 1 file path argument");
        std::process::exit(1);
    }
    let source: Vec<u8> = std::fs::read_to_string(&args[1]).unwrap().into_bytes();

    let mut lexer = Lexer::new(source);

    let start = Instant::now();
    let tokens = match lexer.lex_text() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("{}", format!("Error while lexing: {e}").red());
            std::process::exit(1);
        }
    };
    let elapsed = start.elapsed();

    for token in &tokens {
        let start = token.start as usize;
        let end = token.end as usize;
        let lexeme = String::from_utf8_lossy(&lexer.source[start..end]);

        println!(
            "{}({} {})",
            format!("Token").bright_cyan(),
            format!("{:?}", token.token_type).bright_yellow(),
            format!("{:?}", lexeme).bright_green()
        );
    }

    println!(
        "{}",
        format!("Elapsed: {:?}", elapsed).bright_purple().italic()
    );
}
