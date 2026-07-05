mod lexer;
mod token;

fn main() {
    let mut lexer = lexer::Lexer::new(b"Test");

    let character = lexer.peek_and_advance().expect("Failed to peek");

    println!("{} {}", character as char, lexer.current)
}
