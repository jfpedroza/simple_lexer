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

    fn look_ahead(&self, count: usize) -> OptToken<'a> {
        self.input.get(self.position + count)
    }

    fn current(&self) -> OptToken<'a> {
        self.look_ahead(0)
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

    fn check_ahead(&self, token_type: TokenType, count: usize) -> OptToken<'a> {
        match self.look_ahead(count) {
            Some(token) if token.ttype == token_type => Some(token),
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
            _ => panic!(
                "Token of type {:?} and value '{}' passed to token_to_node",
                ttype, value
            ),
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
            _ => panic!(
                "Token of type {:?} and value '{}' passed to token_to_bin_op_node",
                ttype, value
            ),
        };

        ParseNode {
            ntype,
            location: Location(*line, *column),
        }
    }

    fn token_to_assignment_node(
        Token {
            ttype,
            value,
            line,
            column,
        }: &Token<'_>,
        left_child: ParseNode,
        right_child: ParseNode,
    ) -> ParseNode {
        let right_child = Box::new(right_child);
        let ntype = match (ttype, left_child.ntype) {
            (TokenType::Assign, NodeType::Identifier(value)) => {
                NodeType::Assignment(value, right_child)
            }
            (TokenType::Assign, ntype) => panic!(
                "Left node of type {:?} passed to token_to_assignment_node",
                ntype
            ),
            _ => panic!(
                "Token of type {:?} and value '{}' passed to token_to_assignment_node",
                ttype, value
            ),
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
                self.parse_right_expr()
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

    fn check_assignment_op(&self) -> OptToken {
        self.check_ahead(TokenType::Assign, 1)
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

    fn parse_comp_term(&mut self) -> ParseResult {
        let node = self.parse_term()?;
        let left_child = node.clone();
        self.check_current_in_list(&[TokenType::Plus, TokenType::Minus], true)
            .and_then(|token| {
                let res = self
                    .parse_term()
                    .map(|right_child| Self::token_to_bin_op_node(token, left_child, right_child));
                Some(res)
            })
            .unwrap_or_else(|| Ok(node))
    }

    fn parse_right_expr(&mut self) -> ParseResult {
        let node = self.parse_comp_term()?;
        let left_child = node.clone();
        self.check_current_in_list(
            &[
                TokenType::GreaterThan,
                TokenType::GreaterThanOrEqual,
                TokenType::LessThan,
                TokenType::LessThanOrEqual,
                TokenType::Equal,
            ],
            true,
        )
        .and_then(|token| {
            let res = self
                .parse_comp_term()
                .map(|right_child| Self::token_to_bin_op_node(token, left_child, right_child));
            Some(res)
        })
        .unwrap_or_else(|| Ok(node))
    }

    fn parse_expr(&mut self) -> ParseResult {
        self.parse_identifier(false)
            .and_then(Result::ok)
            .and_then(|id_node| {
                self.check_assignment_op().map(|assign_token| {
                    // We have to return an empty node as right expression because `assign_token`
                    // contains a reference to `self` and it would result in two functions
                    // referencing `self`.
                    Self::token_to_assignment_node(assign_token, id_node, ParseNode::empty_root())
                })
            })
            .map(|node| match node {
                ParseNode {
                    ntype: NodeType::Assignment(id, _),
                    location,
                } => {
                    self.move_forward(2);
                    let right_expr = self.parse_right_expr()?;
                    Ok(ParseNode {
                        ntype: NodeType::Assignment(id, Box::new(right_expr)),
                        location: location,
                    })
                }
                node => panic!("Expected assignment node. Got {:#?}", node),
            })
            .unwrap_or_else(|| self.parse_right_expr())
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

    fn sum_node(
        left_child: ParseNode,
        right_child: ParseNode,
        (line, column): (usize, usize),
    ) -> ParseNode {
        let left_child = Box::new(left_child);
        let right_child = Box::new(right_child);
        ParseNode {
            ntype: NodeType::Sum(left_child, right_child),
            location: Location(line, column),
        }
    }

    fn substraction_node(
        left_child: ParseNode,
        right_child: ParseNode,
        (line, column): (usize, usize),
    ) -> ParseNode {
        let left_child = Box::new(left_child);
        let right_child = Box::new(right_child);
        ParseNode {
            ntype: NodeType::Substraction(left_child, right_child),
            location: Location(line, column),
        }
    }

    fn greater_than_node(
        left_child: ParseNode,
        right_child: ParseNode,
        (line, column): (usize, usize),
    ) -> ParseNode {
        let left_child = Box::new(left_child);
        let right_child = Box::new(right_child);
        ParseNode {
            ntype: NodeType::GreaterThan(left_child, right_child),
            location: Location(line, column),
        }
    }

    fn greater_than_equal_node(
        left_child: ParseNode,
        right_child: ParseNode,
        (line, column): (usize, usize),
    ) -> ParseNode {
        let left_child = Box::new(left_child);
        let right_child = Box::new(right_child);
        ParseNode {
            ntype: NodeType::GreaterThanOrEqual(left_child, right_child),
            location: Location(line, column),
        }
    }

    fn less_than_node(
        left_child: ParseNode,
        right_child: ParseNode,
        (line, column): (usize, usize),
    ) -> ParseNode {
        let left_child = Box::new(left_child);
        let right_child = Box::new(right_child);
        ParseNode {
            ntype: NodeType::LessThan(left_child, right_child),
            location: Location(line, column),
        }
    }

    fn less_than_equal_node(
        left_child: ParseNode,
        right_child: ParseNode,
        (line, column): (usize, usize),
    ) -> ParseNode {
        let left_child = Box::new(left_child);
        let right_child = Box::new(right_child);
        ParseNode {
            ntype: NodeType::LessThanOrEqual(left_child, right_child),
            location: Location(line, column),
        }
    }

    fn equal_node(
        left_child: ParseNode,
        right_child: ParseNode,
        (line, column): (usize, usize),
    ) -> ParseNode {
        let left_child = Box::new(left_child);
        let right_child = Box::new(right_child);
        ParseNode {
            ntype: NodeType::Equal(left_child, right_child),
            location: Location(line, column),
        }
    }

    fn assignment_node(
        identifier: String,
        right_child: ParseNode,
        (line, column): (usize, usize),
    ) -> ParseNode {
        let right_child = Box::new(right_child);
        ParseNode {
            ntype: NodeType::Assignment(identifier, right_child),
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
    fn test_parse_mutli_div_parens() {
        let tokens = Lexer::get_tokens("3.14 * (hello / world)").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Ok(wrap(multiplication_node(
                number_node(3.14f64, (0, 0)),
                division_node(
                    identifier_node("hello", (0, 8)),
                    identifier_node("world", (0, 16)),
                    (0, 14)
                ),
                (0, 5)
            ))),
            parser.parse()
        );
    }

    #[test]
    fn test_parse_sum() {
        let tokens = Lexer::get_tokens("3.14 + hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Ok(wrap(sum_node(
                number_node(3.14f64, (0, 0)),
                identifier_node("hello", (0, 7)),
                (0, 5)
            ))),
            parser.parse()
        );
    }

    #[test]
    fn test_parse_substraction() {
        let tokens = Lexer::get_tokens("3.14 - hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Ok(wrap(substraction_node(
                number_node(3.14f64, (0, 0)),
                identifier_node("hello", (0, 7)),
                (0, 5)
            ))),
            parser.parse()
        );
    }

    #[test]
    fn test_parse_sum_multi() {
        let tokens = Lexer::get_tokens("3.14 + hello * world").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Ok(wrap(sum_node(
                number_node(3.14f64, (0, 0)),
                multiplication_node(
                    identifier_node("hello", (0, 7)),
                    identifier_node("world", (0, 15)),
                    (0, 13)
                ),
                (0, 5)
            ))),
            parser.parse()
        );
    }

    #[test]
    fn test_parse_div_substraction() {
        let tokens = Lexer::get_tokens("3.14 / (hello) - world").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Ok(wrap(substraction_node(
                division_node(
                    number_node(3.14f64, (0, 0)),
                    identifier_node("hello", (0, 8)),
                    (0, 5)
                ),
                identifier_node("world", (0, 17)),
                (0, 15)
            ))),
            parser.parse()
        );
    }

    #[test]
    fn test_parse_greater_than() {
        let tokens = Lexer::get_tokens("3.14 > hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Ok(wrap(greater_than_node(
                number_node(3.14f64, (0, 0)),
                identifier_node("hello", (0, 7)),
                (0, 5)
            ))),
            parser.parse()
        );
    }

    #[test]
    fn test_parse_greater_than_equal() {
        let tokens = Lexer::get_tokens("3.14 >= hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Ok(wrap(greater_than_equal_node(
                number_node(3.14f64, (0, 0)),
                identifier_node("hello", (0, 8)),
                (0, 5)
            ))),
            parser.parse()
        );
    }

    #[test]
    fn test_parse_less_than() {
        let tokens = Lexer::get_tokens("3.14 < hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Ok(wrap(less_than_node(
                number_node(3.14f64, (0, 0)),
                identifier_node("hello", (0, 7)),
                (0, 5)
            ))),
            parser.parse()
        );
    }

    #[test]
    fn test_parse_less_than_equal() {
        let tokens = Lexer::get_tokens("3.14 <= hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Ok(wrap(less_than_equal_node(
                number_node(3.14f64, (0, 0)),
                identifier_node("hello", (0, 8)),
                (0, 5)
            ))),
            parser.parse()
        );
    }

    #[test]
    fn test_parse_equal() {
        let tokens = Lexer::get_tokens("3.14 == hello").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Ok(wrap(equal_node(
                number_node(3.14f64, (0, 0)),
                identifier_node("hello", (0, 8)),
                (0, 5)
            ))),
            parser.parse()
        );
    }

    #[test]
    fn test_parse_less_than_sum() {
        let tokens = Lexer::get_tokens("3.14 < hello + world").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Ok(wrap(less_than_node(
                number_node(3.14f64, (0, 0)),
                sum_node(
                    identifier_node("hello", (0, 7)),
                    identifier_node("world", (0, 15)),
                    (0, 13)
                ),
                (0, 5)
            ))),
            parser.parse()
        );
    }

    #[test]
    fn test_parse_equal_sub_multi() {
        let tokens = Lexer::get_tokens("3.14 - 2 == hello * world").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Ok(wrap(equal_node(
                substraction_node(
                    number_node(3.14f64, (0, 0)),
                    number_node(2f64, (0, 7)),
                    (0, 5)
                ),
                multiplication_node(
                    identifier_node("hello", (0, 12)),
                    identifier_node("world", (0, 20)),
                    (0, 18)
                ),
                (0, 9)
            ))),
            parser.parse()
        );
    }

    #[test]
    fn test_parse_greater_than_sub() {
        let tokens = Lexer::get_tokens("3.14 >= (hello - world)").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Ok(wrap(greater_than_equal_node(
                number_node(3.14f64, (0, 0)),
                substraction_node(
                    identifier_node("hello", (0, 9)),
                    identifier_node("world", (0, 17)),
                    (0, 15)
                ),
                (0, 5)
            ))),
            parser.parse()
        );
    }

    #[test]
    fn test_parse_assignment() {
        let tokens = Lexer::get_tokens("pi = 3.14").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Ok(wrap(assignment_node(
                String::from("pi"),
                number_node(3.14f64, (0, 5)),
                (0, 3)
            ))),
            parser.parse()
        );
    }

    #[test]
    fn test_parse_assignment2() {
        let tokens = Lexer::get_tokens("resp = (hello - world)").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Ok(wrap(assignment_node(
                String::from("resp"),
                substraction_node(
                    identifier_node("hello", (0, 8)),
                    identifier_node("world", (0, 16)),
                    (0, 14)
                ),
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

    #[test]
    fn test_parse_invalid_assignment() {
        let tokens = Lexer::get_tokens("hello =").unwrap();
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            Err(ParsingError::UnexpectedEndOfLine(Location(0, 6))),
            parser.parse()
        );
        assert_eq!(parser.position, 2);
    }
}
