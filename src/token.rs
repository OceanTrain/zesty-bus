// Base type created by the lexer to seperate the program
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
  Text(String),
  Keyword(String),
  Ampersand,
  Asterisk,
  At, 
  Caret, 
  CloseCurlyBrace,
  CloseParen, 
  CloseSquareBracket, 
  Dollar,
  GreaterThan,
  LessThan,
  Minus, 
  OpenCurlyBrace,
  OpenParen, 
  OpenSquareBracket, 
  Percent,
  Pipe,
  Plus,
  Pound,
  Semicolon,
  Tilde,
  Newline,
}

// Contains the type of token as well as its original position in the program
#[derive(Debug, Clone)]
pub struct Token {
  pub kind: TokenKind,
  pub index: usize,
}


