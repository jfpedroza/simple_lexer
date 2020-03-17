use crate::number_fsm;

const ARITHMETIC_OPERATORS: &str = "+-*/";
const COMPARISON_OPERATORS: &str = "=<>";

/// Enumeration of all types of token.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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

#[derive(Eq, PartialEq, Debug)]
pub struct Token<'a> {
    /// - type.   A 'TokenType' corresponding to the type
    ///           of the newly created 'Token'.
    pub ttype: TokenType,

    /// - value.  The 'String' value of the token.
    ///           The actual characters of the lexeme described.
    pub value: &'a str,

    /// - line.   The line number where the token
    ///           was encountered in the source code.
    pub line: usize,

    /// - column. The column number where the token
    ///           was encountered in the source code.
    pub column: usize,
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TokenType::*;
        let type_value = match self.ttype {
            Identifier => format!("Iden({})", self.value),
            Number => format!("Num({})", self.value),
            Plus | Minus | Times | Div => format!("ArOp({})", self.value),
            GreaterThan | GreaterThanOrEqual | LessThan | LessThanOrEqual | Equal => {
                format!("ComOp({})", self.value)
            }
            Assign => format!("Assi({})", self.value),
            LeftParenthesis | RightParenthesis => format!("Paren({})", self.value),
            EndOfInput => String::from("EOI"),
        };

        write!(f, "<{}, {}:{}>", type_value, self.line, self.column)
    }
}

pub struct Lexer<'a> {
    input: &'a str,

    iter: std::iter::Peekable<std::str::Chars<'a>>,

    position: usize,

    line: usize,

    column: usize,
}

#[derive(Debug, PartialEq, Fail)]
pub enum LexingError {
    #[fail(
        display = "Unrecognized character {} at line {} and column {}.",
        character, line, column
    )]
    UnrecognizedCharacter {
        character: char,
        line: usize,
        column: usize,
    },
    #[fail(display = "Invalid number at line {} and column {}.", line, column)]
    InvalidNumber { line: usize, column: usize },
}

type TokenRes<'a> = Result<Token<'a>, LexingError>;

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            iter: input.chars().peekable(),
            position: 0,
            line: 0,
            column: 0,
        }
    }

    pub fn all_tokens(&mut self) -> Result<Vec<Token<'a>>, LexingError> {
        let mut tokens: Vec<Token<'a>> = vec![];
        let mut token_res = self.next_token();
        loop {
            match token_res {
                Ok(Token {
                    ttype: TokenType::EndOfInput,
                    ..
                }) => break,
                Ok(token) => {
                    tokens.push(token);
                    token_res = self.next_token();
                }
                Err(error) => {
                    return Err(error);
                }
            }
        }

        Ok(tokens)
    }

    pub fn get_tokens(input: &'a str) -> Result<Vec<Token<'a>>, LexingError> {
        let mut lexer = Lexer::new(input);
        lexer.all_tokens()
    }

    /// Returns the next recognized 'Token' in the input.
    fn next_token(&mut self) -> TokenRes<'a> {
        // We skip all the whitespaces and new lines in the input.
        self.skip_whitespaces_and_new_lines();

        if self.position >= self.input.len() {
            return Ok(Token {
                ttype: TokenType::EndOfInput,
                value: "",
                line: self.line,
                column: self.column,
            });
        }

        match self.iter.peek() {
            Some(character) if character.is_ascii_alphabetic() => self.recognize_identifier(),
            Some('(') | Some(')') => self.recognize_parenthesis(),
            Some(&op) if ARITHMETIC_OPERATORS.contains(op) => self.recognize_arithmetic_operator(),
            Some(&op) if COMPARISON_OPERATORS.contains(op) => self.recognize_comparison_operator(),
            Some(character) if character.is_digit(10) => self.recognize_number(),
            Some(&character) => Err(unrecognized_character(self, character)),
            None => panic!(
                "Missing expected character in input at line {} and column {}.",
                self.line, self.column
            ),
        }
    }

    fn expect_next(&mut self, message: &str) -> char {
        self.iter.next().unwrap_or_else(|| {
            panic!(
                "Expected {} in input at line {} and column {}.",
                message, self.line, self.column,
            )
        })
    }

    fn recognize_identifier(&mut self) -> TokenRes<'a> {
        let mut size = 0;
        let line = self.line;
        let column = self.column;
        let position = self.position;

        while let Some(&character) = self.iter.peek() {
            if character.is_ascii_alphanumeric() || character == '_' {
                size += 1;
                self.iter.next();
            } else {
                break;
            }
        }

        self.position += size;
        self.column += size;

        Ok(Token {
            ttype: TokenType::Identifier,
            value: &self.input[position..position + size],
            line,
            column,
        })
    }

    fn recognize_parenthesis(&mut self) -> TokenRes<'a> {
        let Self { line, column, .. } = *self;

        let character = self.expect_next("parenthesis");

        let (ttype, value) = if character == '(' {
            (TokenType::LeftParenthesis, "(")
        } else {
            (TokenType::RightParenthesis, ")")
        };

        self.position += 1;
        self.column += 1;

        Ok(Token {
            ttype,
            value,
            line,
            column,
        })
    }

    fn recognize_arithmetic_operator(&mut self) -> TokenRes<'a> {
        let line = self.line;
        let column = self.column;
        let position = self.position;

        self.expect_next("arithmetic operator");
        self.position += 1;
        self.column += 1;

        let value = &self.input[position..position + 1];

        Ok(Token {
            ttype: Self::match_token_type(value),
            value,
            line,
            column,
        })
    }

    fn recognize_comparison_operator(&mut self) -> TokenRes<'a> {
        let line = self.line;
        let column = self.column;
        let position = self.position;

        self.expect_next("comparison operator");

        let value = if let Some('=') = self.iter.peek() {
            self.iter.next();
            self.position += 2;
            self.column += 2;
            &self.input[position..position + 2]
        } else {
            self.position += 1;
            self.column += 1;

            &self.input[position..position + 1]
        };

        Ok(Token {
            ttype: Self::match_token_type(value),
            value,
            line,
            column,
        })
    }

    fn recognize_number(&mut self) -> TokenRes<'a> {
        let line = self.line;
        let column = self.column;
        let position = self.position;

        let fsm = number_fsm::build_number_recognizer();
        let fsm_input = &self.input[position..];

        if let Some(number) = fsm.run(&fsm_input) {
            let size = number.len();
            self.position += size;
            self.column += size;
            for _ in 0..size {
                self.iter.next();
            }

            Ok(Token {
                ttype: TokenType::Number,
                value: number,
                line,
                column,
            })
        } else {
            Err(invalid_number(&self))
        }
    }

    fn match_token_type(value: &str) -> TokenType {
        match value {
            // Arithmetic operators
            "+" => TokenType::Plus,
            "-" => TokenType::Minus,
            "*" => TokenType::Times,
            "/" => TokenType::Div,

            // Comparison operators
            ">" => TokenType::GreaterThan,
            ">=" => TokenType::GreaterThanOrEqual,
            "<" => TokenType::LessThan,
            "<=" => TokenType::LessThanOrEqual,
            "==" => TokenType::Equal,

            // Assignment operator
            "=" => TokenType::Assign,

            // Parenthesis
            "(" => TokenType::LeftParenthesis,
            ")" => TokenType::RightParenthesis,

            _ => panic!("Operator {} not found in match token type.", value),
        }
    }

    fn skip_whitespaces_and_new_lines(&mut self) {
        while let Some(&character) = self.iter.peek() {
            if !character.is_ascii_whitespace() {
                break;
            }

            self.iter.next();
            self.position += 1;

            if character == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
        }
    }
}

fn unrecognized_character(lexer: &Lexer, character: char) -> LexingError {
    LexingError::UnrecognizedCharacter {
        character,
        line: lexer.line,
        column: lexer.column,
    }
}

fn invalid_number(lexer: &Lexer) -> LexingError {
    LexingError::InvalidNumber {
        line: lexer.line,
        column: lexer.column,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use LexingError::*;

    #[test]
    fn test_empty_input() {
        let tokens = Lexer::get_tokens("");
        assert_eq!(Ok(vec![]), tokens);
    }

    fn token_for_identifier(identifier: &str, column: usize) -> Token {
        Token {
            ttype: TokenType::Identifier,
            value: identifier,
            line: 0,
            column,
        }
    }

    #[test]
    fn test_identifier_only_letters() {
        let tokens = Lexer::get_tokens("hello");
        let expected_token = token_for_identifier("hello", 0);
        assert_eq!(Ok(vec![expected_token]), tokens);
    }

    #[test]
    fn test_identifier_with_underscore() {
        let tokens = Lexer::get_tokens("hello_world");
        let expected_token = token_for_identifier("hello_world", 0);
        assert_eq!(Ok(vec![expected_token]), tokens);
    }

    #[test]
    fn test_identifier_with_digits() {
        let tokens = Lexer::get_tokens("h3ll0");
        let expected_token = token_for_identifier("h3ll0", 0);
        assert_eq!(Ok(vec![expected_token]), tokens);
    }

    #[test]
    fn test_full_identifier() {
        let tokens = Lexer::get_tokens("h3llo_w0rld");
        let expected_token = token_for_identifier("h3llo_w0rld", 0);
        assert_eq!(Ok(vec![expected_token]), tokens);
    }

    fn left_paren(column: usize) -> Token<'static> {
        Token {
            ttype: TokenType::LeftParenthesis,
            value: "(",
            line: 0,
            column,
        }
    }

    fn right_paren(column: usize) -> Token<'static> {
        Token {
            ttype: TokenType::RightParenthesis,
            value: ")",
            line: 0,
            column,
        }
    }

    #[test]
    fn test_single_left_paren() {
        let tokens = Lexer::get_tokens("(");
        let expected_token = left_paren(0);
        assert_eq!(Ok(vec![expected_token]), tokens);
    }

    #[test]
    fn test_single_right_paren() {
        let tokens = Lexer::get_tokens(")");
        let expected_token = right_paren(0);
        assert_eq!(Ok(vec![expected_token]), tokens);
    }

    #[test]
    fn test_couple_paren() {
        let tokens = Lexer::get_tokens("()");
        assert_eq!(Ok(vec![left_paren(0), right_paren(1)]), tokens);
    }

    #[test]
    fn test_inverted_couple_paren() {
        let tokens = Lexer::get_tokens(")(");
        assert_eq!(Ok(vec![right_paren(0), left_paren(1)]), tokens);
    }

    #[test]
    fn test_identifier_inside_paren() {
        let tokens = Lexer::get_tokens("(hello_world)");
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
        let tokens = Lexer::get_tokens("hello_world()");
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
        let tokens = Lexer::get_tokens("hello)(world");
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
        let ttype = Lexer::match_token_type(op);
        let token = Token {
            ttype,
            value: op,
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
            let tokens = Lexer::get_tokens(op);
            let (expected_token, _) = an_operator(op, 0);
            assert_eq!(Ok(vec![expected_token]), tokens);
        }

        let tokens = Lexer::get_tokens(ARITHMETIC_OPERATORS);
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
            let tokens = Lexer::get_tokens(op);
            let (expected_token, _) = an_operator(op, 0);
            assert_eq!(Ok(vec![expected_token]), tokens);
        }
    }

    #[test]
    fn test_combination1() {
        let tokens = Lexer::get_tokens("=(hello>=<world+");
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

    fn a_number(num: &str, column: usize) -> (Token, usize) {
        let token = Token {
            ttype: TokenType::Number,
            value: num,
            line: 0,
            column,
        };

        (token, column + num.len())
    }

    #[test]
    fn test_a_single_number() {
        let numbers = ["5", "2.37", "83e2", "4E57", "91.5e4", "2.83e+3", "3E+7"];
        for number in numbers.iter() {
            let tokens = Lexer::get_tokens(number);
            let expected_token = a_number(number, 0).0;
            assert_eq!(Ok(vec![expected_token]), tokens);
        }
    }

    #[test]
    fn test_invalid_numbers() {
        let numbers = ["2.", "83e", "4E", "91.e4"];
        for number in numbers.iter() {
            let tokens = Lexer::get_tokens(number);
            assert_eq!(Err(InvalidNumber { line: 0, column: 0 }), tokens);
        }
    }

    #[test]
    fn test_combination2() {
        let tokens = Lexer::get_tokens("pi=3.1416");
        assert_eq!(
            Ok(vec![
                token_for_identifier("pi", 0),
                an_operator("=", 2).0,
                a_number("3.1416", 3).0,
            ]),
            tokens
        );
    }

    #[test]
    fn test_combination3() {
        let tokens = Lexer::get_tokens("3.1416*7");
        assert_eq!(
            Ok(vec![
                a_number("3.1416", 0).0,
                an_operator("*", 6).0,
                a_number("7", 7).0,
            ]),
            tokens
        );
    }

    #[test]
    fn test_whitespaces1() {
        let tokens = Lexer::get_tokens("hello ) ( world  ");
        assert_eq!(
            Ok(vec![
                token_for_identifier("hello", 0),
                right_paren(6),
                left_paren(8),
                token_for_identifier("world", 10)
            ]),
            tokens
        );
    }

    #[test]
    fn test_whitespaces2() {
        let tokens = Lexer::get_tokens(" 3.1416\n*\t7");
        let (t1, t2, t3) = {
            let (t1, _col) = a_number("3.1416", 1);
            let (mut t2, _col) = an_operator("*", 0);
            let (mut t3, _col) = a_number("7", 2);

            t2.line = 1;
            t3.line = 1;
            (t1, t2, t3)
        };

        assert_eq!(Ok(vec![t1, t2, t3]), tokens);
    }

    #[test]
    fn test_error1() {
        let tokens = Lexer::get_tokens("invalid_character&");
        assert_eq!(
            Err(UnrecognizedCharacter {
                character: '&',
                line: 0,
                column: 17
            }),
            tokens
        );
    }

    #[test]
    fn test_error2() {
        let tokens = Lexer::get_tokens("var = 3\npi=3.14.");
        assert_eq!(
            Err(UnrecognizedCharacter {
                character: '.',
                line: 1,
                column: 7
            }),
            tokens
        );
    }

    #[test]
    fn test_error3() {
        let tokens = Lexer::get_tokens("var = 3\npi=3.14e+ - 8");
        assert_eq!(Err(InvalidNumber { line: 1, column: 3 }), tokens);
    }
}
