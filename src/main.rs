extern crate simple_lexer;

use simple_lexer::*;

fn main() {
    let fsm = identifier_fsm();

    print_test(&fsm, "camelCaseIdentifier"); // => true
    print_test(&fsm, "snake_case_identifier"); // => true
    print_test(&fsm, "_identifierStartingWithUnderscore"); // => true
    print_test(&fsm, "1dentifier_starting_with_digit"); // => false
    print_test(&fsm, "ident1f1er_cont4ining_d1g1ts"); // => true
}

fn print_test<T: Eq + std::hash::Hash + Copy>(fsm: &FSM<T>, input: &str) {
    println!("{} => {}", input, fsm.run(input));
}
