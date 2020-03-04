extern crate simple_lexer;

use simple_lexer::*;

fn main() {
    let input = std::env::args().skip(1).collect::<Vec<_>>().join("\n");

    match Lexer::new(&input).all_tokens() {
        Ok(tokens) => {
            for token in &tokens {
                print!("{} ", token)
            }

            println!();

            match Parser::new(&tokens).parse() {
                Ok(_) => println!("All good!"),
                Err(message) => println!("{}", message),
            }
        }
        Err(message) => println!("{}", message),
    }
}
