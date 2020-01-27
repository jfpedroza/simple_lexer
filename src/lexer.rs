/// Enumeration of all types of token.
#[derive(Clone)]
enum TokenType {
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

#[derive(Clone)]
struct Token {
    /// - type.   A 'TokenType' corresponding to the type
    ///           of the newly created 'Token'.
    ttype: TokenType,

    /// - value.  The 'String' value of the token.
    ///           The actual characters of the lexeme described.
    value: String,

    /// - line.   The line number where the token
    ///           was encountered in the source code.
    line: usize,

    /// - column. The column number where the token
    ///           was encountered in the source code.
    column: usize,
}

pub struct Lexer<'a> {
    input: String,

    iter: std::iter::Peekable<std::str::Chars<'a>>,

    position: usize,

    line: usize,

    column: usize,
}

impl Token {
    fn new(ttype: TokenType) -> Self {
        Token {
            ttype,
            value: String::new(),
            line: 0,
            column: 0,
        }
    }
}

impl Lexer<'_> {
    pub fn new(input: &'static str) -> Self {
        Lexer {
            input: input.to_string(),
            iter: input.chars().peekable(),
            position: 0,
            line: 0,
            column: 0,
        }
    }

    // Returns the next recognized 'Token' in the input.
    fn next_token(&mut self) -> Result<Token, String> {
        if self.position >= self.input.len() {
            return Ok(Token::new(TokenType::EndOfInput));
        }

        // let character = self.input.char_at(self.position);
        match self.iter.peek() {
            Some(character) if character.is_ascii_alphabetic() => self.recognize_identifier(),
            Some(_) => Err(String::from("Error")),
            None => Err(String::from("Missing character in input")),
        }
    }

    fn all_token(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens: Vec<Result<Token, String>> = vec![];
        let mut token = self.next_token();
        loop {
            match token {
                Ok(Token {
                    ttype: TokenType::EndOfInput,
                    ..
                }) => break,
                Ok(_) => {
                    tokens.push(token);
                    token = self.next_token();
                }
                Err(_) => {
                    tokens.push(token);
                    break;
                }
            }
        }

        tokens.iter().cloned().collect()
    }

    fn recognize_identifier(&self) -> Result<Token, String> {
        Err("WIP".to_string())
    }
}
