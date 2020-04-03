use crate::parser::{Location, NodeType, ParseNode};
use std::collections::HashMap;
use std::f64::EPSILON;

#[derive(Debug, PartialEq, Fail)]
pub enum EvalError {
    #[fail(display = "Unimplemented: {}", _0)]
    Unimplemented(String),
    #[fail(display = "Symbol '{}' not found at {:?}", _0, _1)]
    SymbolNotFound(String, Location),
}

type EvalResult = Result<f64, EvalError>;
type SymbolTable = HashMap<String, f64>;

pub struct EvalContext {
    syms: SymbolTable,
}

impl EvalContext {
    pub fn eval(&mut self, node: &ParseNode) -> EvalResult {
        use NodeType::*;
        match &node.ntype {
            Number(num) => Ok(*num),
            Sum(left, right) => self.perform_arithmetic_op(left, right, |l, r| l + r),
            Substraction(left, right) => self.perform_arithmetic_op(left, right, |l, r| l - r),
            Multiplication(left, right) => self.perform_arithmetic_op(left, right, |l, r| l * r),
            Division(left, right) => self.perform_arithmetic_op(left, right, |l, r| l / r),
            GreaterThan(left, right) => self.perform_comparison_op(left, right, |l, r| l > r),
            GreaterThanOrEqual(left, right) => {
                self.perform_comparison_op(left, right, |l, r| l >= r)
            }
            LessThan(left, right) => self.perform_comparison_op(left, right, |l, r| l < r),
            LessThanOrEqual(left, right) => self.perform_comparison_op(left, right, |l, r| l <= r),
            Equal(left, right) => {
                self.perform_comparison_op(left, right, |l, r| (l - r).abs() < EPSILON)
            }
            Assignment(identifier, right) => {
                let val = self.eval(right)?;
                self.syms.insert(identifier.clone(), val);
                Ok(val)
            }
            Identifier(identifier) => self
                .syms
                .get(identifier)
                .map(|val| *val)
                .ok_or_else(|| EvalError::SymbolNotFound(identifier.clone(), node.location)),
            _ => Err(EvalError::Unimplemented(format!(
                "Eval for type {:?}",
                node.ntype
            ))),
        }
    }

    pub fn eval_and_print(root: &ParseNode) -> Result<(), EvalError> {
        let nodes = match &root.ntype {
            NodeType::Root(nodes) => nodes,
            _ => panic!("Expected Root node, got {:?}", root),
        };

        let mut ctx = EvalContext {
            syms: HashMap::new(),
        };

        ctx.populate_symbol_table();

        println!();

        for node in nodes {
            let res = ctx.eval(node)?;
            println!("{}", res);
        }

        Ok(())
    }

    fn perform_arithmetic_op<F>(
        &mut self,
        left_child: &ParseNode,
        right_child: &ParseNode,
        op: F,
    ) -> EvalResult
    where
        F: FnOnce(f64, f64) -> f64,
    {
        let left_res = self.eval(&left_child)?;
        let right_res = self.eval(&right_child)?;
        Ok(op(left_res, right_res))
    }

    fn perform_comparison_op<F>(
        &mut self,
        left_child: &ParseNode,
        right_child: &ParseNode,
        op: F,
    ) -> EvalResult
    where
        F: FnOnce(f64, f64) -> bool,
    {
        let left_res = self.eval(&left_child)?;
        let right_res = self.eval(&right_child)?;
        let res = op(left_res, right_res);
        Ok(if res { 1.0 } else { 0.0 })
    }

    fn populate_symbol_table(&mut self) {
        self.syms.insert(String::from("PI"), std::f64::consts::PI);
    }
}
