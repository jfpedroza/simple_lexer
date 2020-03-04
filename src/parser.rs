use crate::lexer::Token;

type Child = Box<ParseNode>;

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

pub struct ParseNode {
    ntype: NodeType,
    location: (usize, usize),
}

pub struct Parser<'a> {
    iter: std::slice::Iter<'a, Token<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a Vec<Token<'a>>) -> Self {
        Parser { iter: input.iter() }
    }

    pub fn parse(&self) -> Result<ParseNode, String> {
        Err(String::from("Parse Error"))
    }
}
