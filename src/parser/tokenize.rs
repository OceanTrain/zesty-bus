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

pub fn token_kind_to_string(kind: &TokenKind) -> String {
  match kind {
    TokenKind::Text(t) => format!("{}", t),
    TokenKind::Keyword(t) => format!("{}", t),
    TokenKind::Ampersand => format!("&"),
    TokenKind::Asterisk => format!("*"),
    TokenKind::At => format!("@"),
    TokenKind::Caret => format!("^"),
    TokenKind::CloseCurlyBrace => format!("}}"),
    TokenKind::CloseParen => format!(")"),
    TokenKind::CloseSquareBracket => format!("]"),
    TokenKind::Dollar => format!("$"),
    TokenKind::GreaterThan => format!(">"),
    TokenKind::LessThan => format!("<"),
    TokenKind::Minus => format!("-"),
    TokenKind::OpenCurlyBrace => format!("{{"),
    TokenKind::OpenParen => format!("("),
    TokenKind::OpenSquareBracket => format!("["),
    TokenKind::Percent => format!("%"),
    TokenKind::Pipe => format!("|"),
    TokenKind::Plus => format!("+"),
    TokenKind::Pound => format!("#"),
    TokenKind::Semicolon => format!(";"),
    TokenKind::Tilde => format!("~"),
    TokenKind::Newline => format!("\n"),
  }
}

// Basic contructor for Token
impl Token {
  pub fn new(tok: TokenKind, i: usize) -> Token {
    Token {
      kind: tok,
      index: i,
    }
  }

  pub fn to_string(&self) -> String {
    token_kind_to_string(&self.kind)
  }
}

#[allow(dead_code)]
pub fn match_token_kind(kind_a: &TokenKind, kind_b: &TokenKind) -> bool {
  match (kind_a, kind_b) {
    (TokenKind::Text(t_a), TokenKind::Text(t_b)) => if t_a == t_b { return true } else { return false },
    (TokenKind::Keyword(t_a), TokenKind::Keyword(t_b)) => if t_a == t_b { return true } else { return false },
    _ => if kind_a == kind_b { return true } else { return false },
  }
}

pub fn tokens_to_string(tokens: &Vec<Token>) -> String {
  tokens.into_iter().map(|t| -> String { t.to_string() }).collect::<String>()
}


