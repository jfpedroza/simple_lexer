extern crate simple_lexer;

use simple_lexer::*;

fn main() {
    std::env::args()
        .skip(1)
        .for_each(|input| match Lexer::new(&input).all_tokens() {
            Ok(tokens) => {
                for token in tokens {
                    print!("{} ", token)
                }

                println!();
            }
            Err(message) => println!("{}", message),
        });
}
