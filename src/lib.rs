mod fsm;
mod lexer;
pub use fsm::FSM;
pub use lexer::Lexer;

use std::collections::HashSet;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum IdentifierFSMState {
    InitalState,
    AlphanumericOrUnderscore,
}

use IdentifierFSMState::*;

pub fn identifier_fsm() -> FSM<IdentifierFSMState> {
    FSM {
        states: [InitalState, AlphanumericOrUnderscore]
            .iter()
            .cloned()
            .collect(),
        initial_state: InitalState,
        accepting_states: {
            let mut states = HashSet::new();
            states.insert(AlphanumericOrUnderscore);

            states
        },
        next_state: Box::new(|current_state, character| {
            match current_state {
                InitalState => {
                    if character.is_ascii_alphabetic() || character == '_' {
                        return Some(AlphanumericOrUnderscore);
                    }
                }
                AlphanumericOrUnderscore => {
                    if character.is_ascii_alphanumeric() || character == '_' {
                        return Some(AlphanumericOrUnderscore);
                    }
                }
            }

            None
        }),
    }
}
