mod lexer;
mod token;

fn main() {
    let mut lexer = lexer::Lexer::new(b"Test");

    lexer.advance_by(4).expect("Failed to advance");

    let character = lexer.peek().expect("Failed to peek");

    println!("{}", character as char)
}
