use std::collections::HashSet;

pub struct FSM<T: std::cmp::Eq + std::hash::Hash> {
    pub states: HashSet<T>,
    pub initial_state: T,
    pub accepting_states: HashSet<T>,
    pub next_state: Box<dyn Fn(T, char) -> Option<T>>,
}

impl<T: Eq + std::hash::Hash + Copy> FSM<T> {
    /// Runs this FSM on the specified 'input' string.
    /// Returns 'true' if 'input' or a subset of 'input' matches
    /// the regular expression corresponding to this FSM.
    pub fn run(&self, input: &str) -> bool {
        let mut current_state = self.initial_state;

        for character in input.chars() {
            let next_state_fn = &self.next_state;

            match next_state_fn(current_state, character) {
                Some(ref next_state) => {
                    if self.accepting_states.contains(next_state) {
                        return true;
                    }

                    current_state = *next_state
                }
                None => return false,
            }
        }

        self.accepting_states.contains(&current_state)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
    enum IdentifierFSMState {
        InitalState,
        AlphanumericOrUnderscore,
    }

    use IdentifierFSMState::*;

    fn identifier_fsm() -> FSM<IdentifierFSMState> {
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

    #[test]
    fn test_camel_case_identifier() {
        let fsm = identifier_fsm();
        assert_eq!(fsm.run("camelCaseIdentifier"), true);
    }

    #[test]
    fn test_snake_case_identifier() {
        let fsm = identifier_fsm();
        assert_eq!(fsm.run("snake_case_identifier"), true);
    }

    #[test]
    fn test_identifier_starting_with_underscore() {
        let fsm = identifier_fsm();
        assert_eq!(fsm.run("_identifierStartingWithUnderscore"), true);
    }

    #[test]
    fn test_identifier_starting_with_digit() {
        let fsm = identifier_fsm();
        assert_eq!(fsm.run("1dentifier_starting_with_digit"), false);
    }

    #[test]
    fn test_identifier_conteining_d1gits() {
        let fsm = identifier_fsm();
        assert_eq!(fsm.run("ident1f1er_cont4ining_d1g1ts"), true);
    }
}
