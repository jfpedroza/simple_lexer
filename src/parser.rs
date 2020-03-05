use crate::lexer::{Lexer, Token, TokenType};

type Child = Box<ParseNode>;

#[derive(PartialEq, Debug)]
enum NodeType {
    /// Identifiers and literals
    Identifier(String),
    Number(f64),

    /// Arithmetic operations
    Sum(Child, Child),
    Substraction(Child, Child),
    Multiplication(Child, Child),
    Division(Child, Child),

    /// Comparison operations
    GreaterThan(Child, Child),
    GreaterThanOrEqual(Child, Child),
    LessThan(Child, Child),
    LessThanOrEqual(Child, Child),
    Equal(Child, Child),

    /// Assignment operations
    Assignment(String, Child),

    /// Special node
    Root(Vec<ParseNode>),
}

#[derive(PartialEq, Debug)]
pub struct ParseNode {
    ntype: NodeType,
    location: (usize, usize),
}

pub struct Parser<'a> {
    input: &'a Vec<Token<'a>>,
    position: usize,
}

#[derive(Debug, PartialEq, Fail)]
pub enum ParsingError {
    #[fail(display = "Dummy error")]
    DummyError,
}

type ParseResult = Result<ParseNode, ParsingError>;
type OptToken<'a> = Option<&'a Token<'a>>;

impl<'a> Parser<'a> {
    pub fn new(input: &'a Vec<Token<'a>>) -> Self {
        Parser { input, position: 0 }
    }

    pub fn parse(&self) -> ParseResult {
        Err(ParsingError::DummyError)
    }

    fn current(&self) -> OptToken<'a> {
        self.input.get(self.position)
    }

    fn move_forward(&mut self, count: usize) {
        self.position += count;
    }

    fn advance(&mut self) {
        self.move_forward(1);
    }

    fn parse_number(&mut self) -> Option<ParseNode> {
        match self.current() {
            Some(&Token {
                ttype: TokenType::Number,
                value,
                line,
                column,
                ..
            }) => {
                let ntype = NodeType::Number(value.parse().unwrap());
                self.advance();
                Some(ParseNode {
                    ntype,
                    location: (line, column),
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let mut lexer = Lexer::new("3.14");
        let tokens = lexer.all_tokens().unwrap();
        let mut parser = Parser::new(&tokens);
        assert!(
            Some(ParseNode {
                ntype: NodeType::Number(3.14f64),
                location: (0, 0)
            }) == parser.parse_number()
        )
    }

    #[test]
    fn test_parse_number_non_number() {
        let mut lexer = Lexer::new("hello");
        let tokens = lexer.all_tokens().unwrap();
        let mut parser = Parser::new(&tokens);
        assert!(None == parser.parse_number())
    }
}
