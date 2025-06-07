use crate::parser::lex::Expr;

// The AST node
#[derive(Debug, Clone)]
pub struct ParseNode {
  pub children: Vec<ParseNode>,
  pub value: Expr,
}

// Basic contructor for ParseNode
impl ParseNode {
  pub fn new(expr: Expr) -> ParseNode {
    ParseNode {
      children: Vec::new(),
      value: expr,
    }
  }
}

