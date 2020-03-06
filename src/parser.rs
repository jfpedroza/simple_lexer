use crate::lexer::{Token, TokenType};

type Child = Box<ParseNode>;

#[derive(Clone, PartialEq, Debug)]
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

// TODO: Write a Display implementation to print the parse tree in a better way
#[derive(Clone, PartialEq, Debug)]
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
    #[fail(display = "Expected close parenthesis at '{:?}' got {}", _1, _0)]
    ExpectedCloseParen(String, Location),
}

type ParseResult = Result<ParseNode, ParsingError>;
type OptParseResult = Option<ParseResult>;
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
        self.parse_expr()
            .map(ParseNode::wrap_in_root)
            .and_then(|node| {
                if self.current().is_none() {
                    Ok(node)
                } else {
                    Err(self.create_unexpected_error())
                }
            })
    }

    fn current(&self) -> OptToken<'a> {
        self.input.get(self.position)
    }

    fn last_token(&self) -> &'a Token<'a> {
        &self.input[self.position - 1]
    }

    fn move_forward(&mut self, count: usize) {
        self.position += count;
    }

    fn advance(&mut self) {
        self.move_forward(1);
    }

    fn check_current(&mut self, token_type: TokenType, advance: bool) -> OptToken<'a> {
        match self.current() {
            Some(token) if token.ttype == token_type => {
                if advance {
                    self.advance();
                }
                Some(token)
            }
            _ => None,
        }
    }

    fn token_to_node(
        Token {
            ttype,
            value,
            line,
            column,
        }: &Token<'_>,
    ) -> ParseNode {
        let ntype = match ttype {
            TokenType::Identifier => NodeType::Identifier(value.to_string()),
            TokenType::Number => NodeType::Number(value.parse().unwrap()),
            _ => panic!(format!(
                "Token of type {:?} and value '{}' passed to token_to_node",
                ttype, value
            )),
        };

        ParseNode {
            ntype,
            location: Location(*line, *column),
        }
    }

    fn token_to_bin_op_node(
        Token {
            ttype,
            value,
            line,
            column,
        }: &Token<'_>,
        left_child: ParseNode,
        right_child: ParseNode,
    ) -> ParseNode {
        let left_child = Box::new(left_child);
        let right_child = Box::new(right_child);
        let ntype = match ttype {
            TokenType::Plus => NodeType::Sum(left_child, right_child),
            TokenType::Minus => NodeType::Substraction(left_child, right_child),
            TokenType::Times => NodeType::Multiplication(left_child, right_child),
            TokenType::Div => NodeType::Division(left_child, right_child),
            TokenType::GreaterThan => NodeType::GreaterThan(left_child, right_child),
            TokenType::GreaterThanOrEqual => NodeType::GreaterThanOrEqual(left_child, right_child),
            TokenType::LessThan => NodeType::LessThan(left_child, right_child),
            TokenType::LessThanOrEqual => NodeType::LessThanOrEqual(left_child, right_child),
            TokenType::Equal => NodeType::Equal(left_child, right_child),
            _ => panic!(format!(
                "Token of type {:?} and value '{}' passed to token_to_bin_op_node",
                ttype, value
            )),
        };

        ParseNode {
            ntype,
            location: Location(*line, *column),
        }
    }

    fn parse_number(&mut self, advance: bool) -> OptParseResult {
        self.check_current(TokenType::Number, advance)
            .map(Self::token_to_node)
            .map(Result::Ok)
    }

    fn parse_identifier(&mut self, advance: bool) -> OptParseResult {
        self.check_current(TokenType::Identifier, advance)
            .map(Self::token_to_node)
            .map(Result::Ok)
    }

    fn check_open_paren(&mut self, advance: bool) -> OptToken {
        self.check_current(TokenType::LeftParenthesis, advance)
    }

    fn expect_close_paren(&mut self, node: ParseNode) -> ParseResult {
        self.check_current(TokenType::RightParenthesis, true)
            .map(|_| node)
            .ok_or(self.create_close_paren_error())
    }

    fn parse_expr_in_parens(&mut self, advance: bool) -> OptParseResult {
        match self.check_open_paren(advance) {
            Some(_) => Some(
                self.parse_expr()
                    .and_then(|node| self.expect_close_paren(node)),
            ),
            None => None,
        }
    }

    fn check_current_in_list(&mut self, token_types: &[TokenType], advance: bool) -> OptToken<'a> {
        token_types
            .iter()
            .find_map(|ttype| self.check_current(*ttype, advance))
    }

    fn parse_factor(&mut self) -> ParseResult {
        self.parse_number(true)
            .or_else(|| self.parse_identifier(true))
            .or_else(|| self.parse_expr_in_parens(true))
            .unwrap_or_else(|| Err(self.create_unexpected_error()))
    }

    fn parse_term(&mut self) -> ParseResult {
        let node = self.parse_factor()?;
        let left_child = node.clone();
        self.check_current_in_list(&[TokenType::Times, TokenType::Div], true)
            .and_then(|token| {
                let res = self
                    .parse_factor()
                    .map(|right_child| Self::token_to_bin_op_node(token, left_child, right_child));
                Some(res)
            })
            .unwrap_or_else(|| Ok(node))
    }

    fn parse_expr(&mut self) -> ParseResult {
        self.parse_term()
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
                let last_token = self.last_token();
                ParsingError::UnexpectedEndOfLine(Location(
                    last_token.line,
                    last_token.column + last_token.value.len() - 1,
                ))
            }
        }
    }

    fn create_close_paren_error(&self) -> ParsingError {
        match self.current() {
            Some(Token {
                value,
                line,
                column,
                ..
            }) => ParsingError::ExpectedCloseParen(value.to_string(), Location(*line, *column)),
            None => {
                let last_token = self.last_token();
                ParsingError::ExpectedCloseParen(
                    String::from("EOL"),
                    Location(
                        last_token.line,
                        last_token.column + last_token.value.len() - 1,
                    ),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn wrap(node: ParseNode) -> ParseNode {
        ParseNode::wrap_in_root(node)
    }

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

    fn multiplication_node(
        left_child: ParseNode,
        right_child: ParseNode,
        (line, column): (usize, usize),
    ) -> ParseNode {
        let left_child = Box::new(left_child);
        let right_child = Box::new(right_child);
        ParseNode {
            ntype: NodeType::Multiplication(left_child, right_child),
            location: Location(line, column),
        }
    }

    fn division_node(
        left_child: ParseNode,
        right_child: ParseNode,
        (line, column): (usize, usize),
    ) -> ParseNode {
        let left_child = Box::new(left_child);
        let right_child = Box::new(right_child);
        ParseNode {
            ntype: NodeType::Division(left_child, right_child),
            location: Location(line, column),
        }
    }

    #[test]
    fn test_fn_parse_number() {
        let tokens = Lexer::get_tokens("3.14").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Some(Ok(number_node(3.14f64, (0, 0)))),
            parser.parse_number(true)
        );

        assert_eq!(parser.position, 1);
    }

    #[test]
    fn test_fn_parse_number_non_number() {
        let tokens = Lexer::get_tokens("hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(None, parser.parse_number(true));

        assert_eq!(parser.position, 0);
    }

    #[test]
    fn test_fn_parse_identifier() {
        let tokens = Lexer::get_tokens("hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Some(Ok(identifier_node("hello", (0, 0)))),
            parser.parse_identifier(true)
        );

        assert_eq!(parser.position, 1);
    }

    #[test]
    fn test_fn_parse_identifier_non_identifier() {
        let tokens = Lexer::get_tokens("3.14").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(None, parser.parse_identifier(true));

        assert_eq!(parser.position, 0);
    }

    #[test]
    fn test_fn_parse_factor() {
        let tokens = Lexer::get_tokens("3.14 hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(Ok(number_node(3.14f64, (0, 0))), parser.parse_factor());
        assert_eq!(Ok(identifier_node("hello", (0, 5))), parser.parse_factor());
        assert_eq!(
            Err(ParsingError::UnexpectedEndOfLine(Location(0, 9))),
            parser.parse_factor()
        );
        assert_eq!(parser.position, 2);
    }

    #[test]
    fn test_fn_parse_factor2() {
        let tokens = Lexer::get_tokens("hello + world").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(Ok(identifier_node("hello", (0, 0))), parser.parse_factor());
        assert_eq!(
            Err(ParsingError::UnexpectedToken(
                String::from("+"),
                Location(0, 6)
            )),
            parser.parse_factor()
        );
        assert_eq!(parser.position, 1);
    }

    #[test]
    fn test_fn_expr_in_parens() {
        let tokens = Lexer::get_tokens("(hello)").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(Ok(identifier_node("hello", (0, 1))), parser.parse_expr());
        assert_eq!(parser.position, 3);
    }

    #[test]
    fn test_fn_expr_in_double_parens() {
        let tokens = Lexer::get_tokens("((hello) )").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(Ok(identifier_node("hello", (0, 2))), parser.parse_expr());
        assert_eq!(parser.position, 5);
    }

    #[test]
    fn test_fn_expr_in_unclosed_paren() {
        let tokens = Lexer::get_tokens("(hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Err(ParsingError::ExpectedCloseParen(
                String::from("EOL"),
                Location(0, 5)
            )),
            parser.parse_expr()
        );
        assert_eq!(parser.position, 2);
    }

    #[test]
    fn test_fn_expr_in_unclosed_paren2() {
        let tokens = Lexer::get_tokens("(hello j").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Err(ParsingError::ExpectedCloseParen(
                String::from("j"),
                Location(0, 7)
            )),
            parser.parse_expr()
        );
        assert_eq!(parser.position, 2);
    }

    #[test]
    fn test_parse_empty_input() {
        let tokens = Lexer::get_tokens("").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(Ok(ParseNode::empty_root()), parser.parse());
    }

    #[test]
    fn test_parse_number() {
        let tokens = Lexer::get_tokens("3.14").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(Ok(wrap(number_node(3.14f64, (0, 0)))), parser.parse());
    }

    #[test]
    fn test_parse_identifier() {
        let tokens = Lexer::get_tokens("hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(Ok(wrap(identifier_node("hello", (0, 0)))), parser.parse());
    }

    #[test]
    fn test_parse_expr_in_parens() {
        let tokens = Lexer::get_tokens("(hello)").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(Ok(wrap(identifier_node("hello", (0, 1)))), parser.parse());
    }

    #[test]
    fn test_parse_expr_in_double_parens() {
        let tokens = Lexer::get_tokens("((3.14))").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(Ok(wrap(number_node(3.14f64, (0, 2)))), parser.parse());
    }

    #[test]
    fn test_parse_multiplication() {
        let tokens = Lexer::get_tokens("3.14 * hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Ok(wrap(multiplication_node(
                number_node(3.14f64, (0, 0)),
                identifier_node("hello", (0, 7)),
                (0, 5)
            ))),
            parser.parse()
        );
    }

    #[test]
    fn test_parse_division() {
        let tokens = Lexer::get_tokens("3.14 / hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Ok(wrap(division_node(
                number_node(3.14f64, (0, 0)),
                identifier_node("hello", (0, 7)),
                (0, 5)
            ))),
            parser.parse()
        );
    }

    #[test]
    fn test_parse_trailing_token() {
        let tokens = Lexer::get_tokens("3.14 hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Err(ParsingError::UnexpectedToken(
                String::from("hello"),
                Location(0, 5)
            )),
            parser.parse()
        );
        assert_eq!(parser.position, 1);
    }
}
