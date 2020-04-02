extern crate simple_lexer;

use simple_lexer::*;

fn main() {
    let input = std::env::args().skip(1).collect::<Vec<_>>().join("\n");

    let result = Lexer::new(&input)
        .all_tokens()
        .map(|tokens| {
            println!("Lexer result:");
            for token in tokens.iter() {
                print!("{} ", token)
            }

            println!();
            tokens
        })
        .map_err(|err| err.to_string())
        .and_then(|tokens| Parser::new(&tokens).parse().map_err(|err| err.to_string()))
        .map(|root| {
            println!("Parser result:\n{}", root);
            root
        })
        .and_then(|root| EvalContext::eval_and_print(&root).map_err(|err| err.to_string()));

    match result {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1)
        }
    }
}
