extern crate simple_lexer;

use simple_lexer::*;
use std::collections::HashSet;

fn main() {
    let fsm = FSM {
        states: {
            let mut states = HashSet::new();
            states.insert(1);
            states.insert(2);

            states
        },
        initial_state: 1,
        accepting_states: {
            let mut states = HashSet::new();
            states.insert(2);

            states
        },
        next_state: Box::new(|current_state, character| {
            match current_state {
                1 => {
                    if character.is_ascii_alphabetic() || character == '_' {
                        return Some(2);
                    }
                }
                2 => {
                    if character.is_ascii_alphanumeric() || character == '_' {
                        return Some(2);
                    }
                }
                _ => (),
            }

            None
        }),
    };

    print_test(&fsm, "camelCaseIdentifier"); // => true
    print_test(&fsm, "snake_case_identifier"); // => true
    print_test(&fsm, "_identifierStartingWithUnderscore"); // => true
    print_test(&fsm, "1dentifier_starting_with_digit"); // => false
    print_test(&fsm, "ident1f1er_cont4ining_d1g1ts"); // => true
}

fn print_test(fsm: &FSM, input: &str) {
    println!("{} => {}", input, fsm.run(input));
}
