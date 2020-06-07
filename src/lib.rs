extern crate failure;
#[macro_use]
extern crate failure_derive;

mod eval;
mod fsm;
mod lexer;
mod number_fsm;
mod parser;

pub use eval::EvalContext;
pub use fsm::FSM;
pub use lexer::Lexer;
pub use parser::Parser;
