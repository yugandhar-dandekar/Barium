use std::time::Instant;

mod lexer;
mod token;

fn main() {
    let line = b"let x = 12345 + variable_name * 3.14;\n";
    let source = line.repeat(1_000_000);

    let mut lexer = lexer::Lexer::new(source);

    let start = Instant::now();
    lexer.lex_text().unwrap();
    let elapsed = start.elapsed();

    println!("Elapsed: {:?}", elapsed);
}
