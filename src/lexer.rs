const ARITHMETIC_OPERATORS: &str = "+-*/";
const COMPARISON_OPERATORS: &str = "=<>";

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

    // TODO: Store the token value as a String slice
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

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
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
            return Ok(Token {
                ttype: TokenType::EndOfInput,
                value: String::new(),
                line: self.line,
                column: self.column,
            });
        }

        match self.iter.peek() {
            Some(character) if character.is_ascii_alphabetic() => self.recognize_identifier(),
            Some('(') | Some(')') => self.recognize_parenthesis(),
            Some(&op) if ARITHMETIC_OPERATORS.contains(op) => self.recognize_arithmetic_operator(),
            Some(&op) if COMPARISON_OPERATORS.contains(op) => self.recognize_comparison_operator(),
            Some(_) => Err(String::from("Error")),
            None => Err(String::from("Missing expected character in input.")),
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

    fn recognize_parenthesis(&mut self) -> Result<Token, String> {
        let line = self.line;
        let column = self.column;

        let character = self.iter.next().ok_or("Expected parenthesis in input.")?;

        let (ttype, value) = if character == '(' {
            (TokenType::LeftParenthesis, "(")
        } else {
            (TokenType::RightParenthesis, ")")
        };

        self.position += 1;
        self.column += 1;

        Ok(Token {
            ttype,
            value: String::from(value),
            line,
            column,
        })
    }

    fn recognize_arithmetic_operator(&mut self) -> Result<Token, String> {
        let line = self.line;
        let column = self.column;

        let character = self
            .iter
            .next()
            .ok_or("Expected arithmetic operator in input.")?;

        self.position += 1;
        self.column += 1;

        let value = character.to_string();

        Ok(Token {
            ttype: Self::match_token_type(&value)?,
            value,
            line,
            column,
        })
    }

    fn recognize_comparison_operator(&mut self) -> Result<Token, String> {
        let line = self.line;
        let column = self.column;

        let character = self
            .iter
            .next()
            .ok_or("Expected comparison operator in input.")?;

        let value = if let Some('=') = self.iter.peek() {
            self.iter.next();
            self.position += 2;
            self.column += 2;
            format!("{}=", character)
        } else {
            self.position += 1;
            self.column += 1;

            character.to_string()
        };

        Ok(Token {
            ttype: Self::match_token_type(&value)?,
            value,
            line,
            column,
        })
    }

    fn match_token_type(value: &str) -> Result<TokenType, String> {
        match value {
            // Arithmetic operators
            "+" => Ok(TokenType::Plus),
            "-" => Ok(TokenType::Minus),
            "*" => Ok(TokenType::Times),
            "/" => Ok(TokenType::Div),

            // Comparison operators
            ">" => Ok(TokenType::GreaterThan),
            ">=" => Ok(TokenType::GreaterThanOrEqual),
            "<" => Ok(TokenType::LessThan),
            "<=" => Ok(TokenType::LessThanOrEqual),
            "==" => Ok(TokenType::Equal),

            // Assignment operator
            "=" => Ok(TokenType::Assign),

            // Parenthesis
            "(" => Ok(TokenType::LeftParenthesis),
            ")" => Ok(TokenType::RightParenthesis),

            _ => Err(format!("Operator {} not found in match token type.", value)),
        }
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

    fn token_for_identifier(identifier: &str, column: usize) -> Token {
        Token {
            ttype: TokenType::Identifier,
            value: String::from(identifier),
            line: 0,
            column,
        }
    }

    #[test]
    fn test_identifier_only_letters() {
        let mut lexer = Lexer::new("hello");
        let tokens = lexer.all_tokens();
        let expected_token = token_for_identifier("hello", 0);
        assert_eq!(Ok(vec![expected_token]), tokens);
    }

    #[test]
    fn test_identifier_with_underscore() {
        let mut lexer = Lexer::new("hello_world");
        let tokens = lexer.all_tokens();
        let expected_token = token_for_identifier("hello_world", 0);
        assert_eq!(Ok(vec![expected_token]), tokens);
    }

    #[test]
    fn test_identifier_with_digits() {
        let mut lexer = Lexer::new("h3ll0");
        let tokens = lexer.all_tokens();
        let expected_token = token_for_identifier("h3ll0", 0);
        assert_eq!(Ok(vec![expected_token]), tokens);
    }

    #[test]
    fn test_full_identifier() {
        let mut lexer = Lexer::new("h3llo_w0rld");
        let tokens = lexer.all_tokens();
        let expected_token = token_for_identifier("h3llo_w0rld", 0);
        assert_eq!(Ok(vec![expected_token]), tokens);
    }

    fn left_paren(column: usize) -> Token {
        Token {
            ttype: TokenType::LeftParenthesis,
            value: String::from("("),
            line: 0,
            column,
        }
    }

    fn right_paren(column: usize) -> Token {
        Token {
            ttype: TokenType::RightParenthesis,
            value: String::from(")"),
            line: 0,
            column,
        }
    }

    #[test]
    fn test_single_left_paren() {
        let mut lexer = Lexer::new("(");
        let tokens = lexer.all_tokens();
        let expected_token = left_paren(0);
        assert_eq!(Ok(vec![expected_token]), tokens);
    }

    #[test]
    fn test_single_right_paren() {
        let mut lexer = Lexer::new(")");
        let tokens = lexer.all_tokens();
        let expected_token = right_paren(0);
        assert_eq!(Ok(vec![expected_token]), tokens);
    }

    #[test]
    fn test_couple_paren() {
        let mut lexer = Lexer::new("()");
        let tokens = lexer.all_tokens();
        assert_eq!(Ok(vec![left_paren(0), right_paren(1)]), tokens);
    }

    #[test]
    fn test_inverted_couple_paren() {
        let mut lexer = Lexer::new(")(");
        let tokens = lexer.all_tokens();
        assert_eq!(Ok(vec![right_paren(0), left_paren(1)]), tokens);
    }

    #[test]
    fn test_identifier_inside_paren() {
        let mut lexer = Lexer::new("(hello_world)");
        let tokens = lexer.all_tokens();
        assert_eq!(
            Ok(vec![
                left_paren(0),
                token_for_identifier("hello_world", 1),
                right_paren(12)
            ]),
            tokens
        );
    }

    #[test]
    fn test_identifier_then_parens() {
        let mut lexer = Lexer::new("hello_world()");
        let tokens = lexer.all_tokens();
        assert_eq!(
            Ok(vec![
                token_for_identifier("hello_world", 0),
                left_paren(11),
                right_paren(12)
            ]),
            tokens
        );
    }

    #[test]
    fn test_ident_parens_ident() {
        let mut lexer = Lexer::new("hello)(world");
        let tokens = lexer.all_tokens();
        assert_eq!(
            Ok(vec![
                token_for_identifier("hello", 0),
                right_paren(5),
                left_paren(6),
                token_for_identifier("world", 7)
            ]),
            tokens
        );
    }

    fn an_operator(op: &str, column: usize) -> (Token, usize) {
        let ttype = Lexer::match_token_type(op).unwrap();
        let token = Token {
            ttype,
            value: String::from(op),
            line: 0,
            column,
        };

        (token, column + op.len())
    }

    #[test]
    fn test_arithmethic_operators() {
        let ops = ARITHMETIC_OPERATORS
            .split("")
            .filter(|op| !op.is_empty())
            .collect::<Vec<_>>();

        for op in ops.iter() {
            let mut lexer = Lexer::new(op);
            let tokens = lexer.all_tokens();
            let (expected_token, _) = an_operator(op, 0);
            assert_eq!(Ok(vec![expected_token]), tokens);
        }

        let mut lexer = Lexer::new(ARITHMETIC_OPERATORS);
        let tokens = lexer.all_tokens();
        let expected_tokens = ops
            .iter()
            .enumerate()
            .map(|(i, op)| an_operator(op, i).0)
            .collect::<Vec<_>>();
        assert_eq!(Ok(expected_tokens), tokens);
    }

    #[test]
    fn test_comparison_operators() {
        let ops = {
            let mut ops = COMPARISON_OPERATORS
                .split("")
                .filter(|op| !op.is_empty())
                .map(|op| op.to_string())
                .collect::<Vec<_>>();

            let mut eq_ops = ops.iter().map(|op| format!("{}=", op)).collect::<Vec<_>>();

            ops.append(&mut eq_ops);

            ops.clone()
        };

        for op in ops.iter() {
            let mut lexer = Lexer::new(op);
            let tokens = lexer.all_tokens();
            let (expected_token, _) = an_operator(op, 0);
            assert_eq!(Ok(vec![expected_token]), tokens);
        }
    }
    #[test]
    fn test_combination1() {
        let mut lexer = Lexer::new("=(hello>=<world+");
        let tokens = lexer.all_tokens();
        assert_eq!(
            Ok(vec![
                an_operator("=", 0).0,
                left_paren(1),
                token_for_identifier("hello", 2),
                an_operator(">=", 7).0,
                an_operator("<", 9).0,
                token_for_identifier("world", 10),
                an_operator("+", 15).0,
            ]),
            tokens
        );
    }
}
