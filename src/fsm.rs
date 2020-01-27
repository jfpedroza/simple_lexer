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
