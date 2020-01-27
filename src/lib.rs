use std::collections::HashSet;

/// Enumeration of all types of token.
pub enum TokenType {
    /// Identifiers and literals
    Identifier,
    Number,

    /// Arithmetic operators
    Plus, // +
    Minus, // -
    Times, // *
    Div,   // /

    /// Comparison operators
    GreaterThan, // >
    GreaterThanOrEqual, // >=
    LessThan,           // <
    LessThanOrEqual,    // <=
    Equal,              // ==

    /// Assignment operator
    Assign, // =

    /// Parenthesis
    LeftParenthesis, // (
    RightParenthesis, // )

    /// Special tokens
    EndOfInput,
}

pub struct Token {
    /// - type.   A 'TokenType' corresponding to the type
    ///           of the newly created 'Token'.
    pub ttype: TokenType,

    /// - value.  The 'String' value of the token.
    ///           The actual characters of the lexeme described.
    pub value: String,

    /// - line.   The line number where the token
    ///           was encountered in the source code.
    pub line: usize,

    /// - column. The column number where the token
    ///           was encountered in the source code.
    pub column: usize,
}

pub struct Lexer {
    pub input: String,

    pub position: usize,

    pub line: usize,

    pub column: usize,
}

pub struct FSM {
    pub states: HashSet<i32>,
    pub initial_state: i32,
    pub accepting_states: HashSet<i32>,
    pub next_state: Box<dyn Fn(i32, char) -> Option<i32>>,
}

impl Token {
    pub fn new(ttype: TokenType) -> Self {
        Token {
            ttype,
            value: String::new(),
            line: 0,
            column: 0,
        }
    }
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.to_string(),
            position: 0,
            line: 0,
            column: 0,
        }
    }

    // Returns the next recognized 'Token' in the input.
    pub fn next_token(&self) -> Result<Token, String> {
        if self.position >= self.input.len() {
            return Ok(Token::new(TokenType::EndOfInput));
        }

        // let character = self.input.char_at(self.position);

        Err("Error".to_string())
    }
}

impl FSM {
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
