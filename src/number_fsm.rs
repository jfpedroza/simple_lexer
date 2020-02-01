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
                BeginNumberWithSignedExponent | NumberWithExponent => {
                    if character.is_digit(10) {
                        return Some(NumberWithExponent);
                    }
                }
            }

            None
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_numbers() {
        let fsm = build_number_recognizer();
        let numbers = ["5", "2.37", "83e2", "4E57", "91.5e4", "2.83e+3", "3E+7"];
        for number in numbers.iter() {
            assert_eq!(Some(*number), fsm.run(number));
        }
    }

    #[test]
    fn test_valid_number_with_extra_data() {
        let fsm = build_number_recognizer();
        let input = "3.1416*2";
        let number = "3.1416";
        assert_eq!(Some(number), fsm.run(input));
    }

    #[test]
    fn test_invalid_numbers() {
        let fsm = build_number_recognizer();
        let numbers = ["l5", "2.", "83e", "4E", "91.e4"];
        for number in numbers.iter() {
            assert_eq!(None, fsm.run(number));
        }
    }
}
