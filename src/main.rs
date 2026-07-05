mod lexer;
mod token;

fn main() {
    let lexer = lexer::Lexer::new(b"Test");

    println!("{}", String::from_utf8_lossy(lexer.source))
}
