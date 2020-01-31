use crate::fsm::FSM;
use std::collections::HashSet;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum NumberFSMState {
    Initial,
    Integer,
    BeginNumberWithFractionalPart,
    NumberWithFractionalPart,
    BeginNumberWithExponent,
    BeginNumberWithSignedExponent,
    NumberWithExponent,
}

impl NumberFSMState {
    fn get_all() -> HashSet<Self> {
        use NumberFSMState::*;
        [
            Initial,
            Integer,
            BeginNumberWithFractionalPart,
            NumberWithFractionalPart,
            BeginNumberWithExponent,
            BeginNumberWithSignedExponent,
            NumberWithExponent,
        ]
        .iter()
        .cloned()
        .collect()
    }
}

pub fn build_number_recognizer() -> FSM<NumberFSMState> {
    use NumberFSMState::*;
    FSM {
        states: NumberFSMState::get_all(),
        initial_state: Initial,
        accepting_states: [Integer, NumberWithFractionalPart, NumberWithExponent]
            .iter()
            .cloned()
            .collect(),
        next_state: Box::new(|current_state, character| {
            match current_state {
                Initial => {
                    if character.is_digit(10) {
                        return Some(Integer);
                    }
                }
                Integer => match character {
                    character if character.is_digit(10) => return Some(Integer),
                    '.' => return Some(BeginNumberWithFractionalPart),
                    'e' | 'E' => return Some(BeginNumberWithExponent),
                    _ => (),
                },
                BeginNumberWithFractionalPart => {
                    if character.is_digit(10) {
                        return Some(NumberWithFractionalPart);
                    }
                }
                NumberWithFractionalPart => match character {
                    character if character.is_digit(10) => return Some(NumberWithFractionalPart),
                    'e' | 'E' => return Some(BeginNumberWithExponent),
                    _ => (),
                },
                BeginNumberWithExponent => match character {
                    character if character.is_digit(10) => return Some(NumberWithExponent),
                    '+' | '-' => return Some(BeginNumberWithSignedExponent),
                    _ => (),
                },
                BeginNumberWithSignedExponent => {
                    if character.is_digit(10) {
                        return Some(NumberWithExponent);
                    }
                }
                _ => (),
            }

            None
        }),
    }
}
