/// Enumeration of all types of token.
#[derive(Clone, Eq, PartialEq, Debug)]
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

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Token {
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

        match self.iter.peek() {
            Some(character) if character.is_ascii_alphabetic() => self.recognize_identifier(),
            Some(_) => Err(String::from("Error")),
            None => Err(String::from("Missing character in input")),
        }
    }

    pub fn all_tokens(&mut self) -> Result<Vec<Token>, String> {
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

    fn recognize_identifier(&mut self) -> Result<Token, String> {
        let mut identifier = vec![];
        let line = self.line;
        let column = self.column;

        while let Some(&character) = self.iter.peek() {
            if character.is_ascii_alphanumeric() || character == '_' {
                identifier.push(character);
                self.iter.next();
            } else {
                break;
            }
        }

        self.position += identifier.len();
        self.column += identifier.len();

        Ok(Token {
            ttype: TokenType::Identifier,
            value: identifier.iter().collect(),
            line,
            column,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("");
        let tokens = lexer.all_tokens();
        assert_eq!(Ok(vec![]), tokens);
    }

    fn token_for_identifier(identifier: &str) -> Token {
        Token {
            ttype: TokenType::Identifier,
            value: String::from(identifier),
            line: 0,
            column: 0,
        }
    }

    #[test]
    fn test_identifier_only_letters() {
        let mut lexer = Lexer::new("hello");
        let tokens = lexer.all_tokens();
        let expected_token = token_for_identifier("hello");
        assert_eq!(Ok(vec![expected_token]), tokens);
    }

    #[test]
    fn test_identifier_with_underscore() {
        let mut lexer = Lexer::new("hello_world");
        let tokens = lexer.all_tokens();
        let expected_token = token_for_identifier("hello_world");
        assert_eq!(Ok(vec![expected_token]), tokens);
    }

    #[test]
    fn test_identifier_with_digits() {
        let mut lexer = Lexer::new("h3ll0");
        let tokens = lexer.all_tokens();
        let expected_token = token_for_identifier("h3ll0");
        assert_eq!(Ok(vec![expected_token]), tokens);
    }

    #[test]
    fn test_full_identifier() {
        let mut lexer = Lexer::new("h3llo_w0rld");
        let tokens = lexer.all_tokens();
        let expected_token = token_for_identifier("h3llo_w0rld");
        assert_eq!(Ok(vec![expected_token]), tokens);
    }
}
