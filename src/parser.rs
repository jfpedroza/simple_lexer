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

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Location(usize, usize);

#[derive(PartialEq, Debug)]
pub struct ParseNode {
    ntype: NodeType,
    location: Location,
}

pub struct Parser<'a> {
    input: &'a Vec<Token<'a>>,
    position: usize,
}

#[derive(Debug, PartialEq, Fail)]
pub enum ParsingError {
    #[fail(display = "Dummy error")]
    DummyError,
    #[fail(display = "Unexpected token '{}' at {:?}", _0, _1)]
    UnexpectedToken(String, Location),
    #[fail(display = "Unexpected end of line: {:?}", _0)]
    UnexpectedEndOfLine(Location),
}

type ParseResult = Result<ParseNode, ParsingError>;
type OptToken<'a> = Option<&'a Token<'a>>;

impl ParseNode {
    fn empty_root() -> Self {
        ParseNode {
            ntype: NodeType::Root(vec![]),
            location: Location(0, 0),
        }
    }

    fn wrap_in_root(node: Self) -> Self {
        let location = node.location;
        ParseNode {
            ntype: NodeType::Root(vec![node]),
            location: location,
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a Vec<Token<'a>>) -> Self {
        Parser { input, position: 0 }
    }

    pub fn parse(&mut self) -> ParseResult {
        if self.input.is_empty() {
            return Ok(ParseNode::empty_root());
        }

        // TODO: Make it work with multple lines
        self.parse_expr().map(ParseNode::wrap_in_root)
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

    fn check_current<F>(&mut self, token_type: TokenType, advance: bool, f: F) -> Option<ParseNode>
    where
        F: FnOnce(&Token<'a>) -> NodeType,
    {
        match self.current() {
            Some(token) if token.ttype == token_type => {
                let ntype = f(token);
                if advance {
                    self.advance();
                }
                Some(ParseNode {
                    ntype,
                    location: Location(token.line, token.column),
                })
            }
            _ => None,
        }
    }

    fn parse_number(&mut self, advance: bool) -> Option<ParseNode> {
        self.check_current(TokenType::Number, advance, |Token { value, .. }| {
            NodeType::Number(value.parse().unwrap())
        })
    }

    fn parse_identifier(&mut self, advance: bool) -> Option<ParseNode> {
        self.check_current(TokenType::Identifier, advance, |Token { value, .. }| {
            NodeType::Identifier(value.to_string())
        })
    }

    fn expect_open_paren(&mut self, advance: bool) -> Option<ParseNode> {
        self.check_current(
            TokenType::LeftParenthesis,
            advance,
            |Token { value, .. }| NodeType::Identifier(value.to_string()),
        )
    }

    fn parse_expr_in_parens(&mut self) -> ParseResult {}

    fn parse_factor(&mut self) -> ParseResult {
        self.parse_number(true)
            .or_else(|| self.parse_identifier(true))
            .ok_or(self.create_unexpected_error())
    }

    fn parse_expr(&mut self) -> ParseResult {
        self.parse_factor()
    }

    fn create_unexpected_error(&self) -> ParsingError {
        match self.current() {
            Some(Token {
                value,
                line,
                column,
                ..
            }) => ParsingError::UnexpectedToken(value.to_string(), Location(*line, *column)),
            None => {
                let last_token = &self.input[self.position - 1];
                ParsingError::UnexpectedEndOfLine(Location(
                    last_token.line,
                    last_token.column + last_token.value.len() - 1,
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn number_node(num: f64, (line, column): (usize, usize)) -> ParseNode {
        ParseNode {
            ntype: NodeType::Number(num),
            location: Location(line, column),
        }
    }

    fn identifier_node(value: &str, (line, column): (usize, usize)) -> ParseNode {
        ParseNode {
            ntype: NodeType::Identifier(String::from(value)),
            location: Location(line, column),
        }
    }

    #[test]
    fn test_parse_number() {
        let tokens = Lexer::get_tokens("3.14").unwrap();
        let mut parser = Parser::new(&tokens);
        assert!(Some(number_node(3.14f64, (0, 0))) == parser.parse_number(true));

        assert_eq!(parser.position, 1);
    }

    #[test]
    fn test_parse_number_non_number() {
        let tokens = Lexer::get_tokens("hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert!(None == parser.parse_number(true));

        assert_eq!(parser.position, 0);
    }

    #[test]
    fn test_parse_identifier() {
        let tokens = Lexer::get_tokens("hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert!(Some(identifier_node("hello", (0, 0))) == parser.parse_identifier(true));

        assert_eq!(parser.position, 1);
    }

    #[test]
    fn test_parse_identifier_non_identifier() {
        let tokens = Lexer::get_tokens("3.14").unwrap();
        let mut parser = Parser::new(&tokens);
        assert!(None == parser.parse_identifier(true));

        assert_eq!(parser.position, 0);
    }

    #[test]
    fn test_parse_factor() {
        let tokens = Lexer::get_tokens("3.14 hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert!(Ok(number_node(3.14f64, (0, 0))) == parser.parse_factor());
        assert!(Ok(identifier_node("hello", (0, 5))) == parser.parse_factor());
        assert!(Err(ParsingError::UnexpectedEndOfLine(Location(0, 9))) == parser.parse_factor());
        assert_eq!(parser.position, 2);
    }

    #[test]
    fn test_parse_factor2() {
        let tokens = Lexer::get_tokens("hello + world").unwrap();
        let mut parser = Parser::new(&tokens);
        assert!(Ok(identifier_node("hello", (0, 0))) == parser.parse_factor());
        assert!(
            Err(ParsingError::UnexpectedToken(
                String::from("+"),
                Location(0, 6)
            )) == parser.parse_factor()
        );
        assert_eq!(parser.position, 1);
    }
}
