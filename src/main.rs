mod lexer;
mod token;

fn main() {
    let lexer = lexer::Lexer::new(b"Test");

    let character = lexer.peek().expect("Failed to peek");

    println!("{}", character as char)
}
