use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

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

// The more complex grammar expressions
#[derive(Debug, Clone)]
pub enum Expr {
  None,
  Room,
}

// Contains the type of token as well as its original position in the program
#[derive(Debug, Clone)]
pub struct Token {
  kind: TokenKind,
  index: usize,
}

// The AST node
#[derive(Debug, Clone)]
pub struct ParseNode {
  pub children: Vec<ParseNode>,
  pub value: Expr,
}

// Holds all of the immortal program information
#[derive(Debug, Clone)]
pub struct Program {
  pub filename: String,
  pub text: Vec<char>,
  pub tokens: Vec<Token>,
  row_index: Vec<usize>,
}

fn token_kind_to_string(kind: &TokenKind) -> String {
  match kind {
    TokenKind::Text(t) => format!("\"{}\"", t),
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
    _ => format!("TODO"),
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

// Basic contructor for ParseNode
impl ParseNode {
  pub fn new() -> ParseNode {
    ParseNode {
      children: Vec::new(),
      value: Expr::None,
    }
  }
}

impl Program {
  pub fn new(name: String, text: Vec<char>) -> Program {
    let mut index = Vec::new();
    for (i, c) in text.iter().enumerate() {
      if *c == '\n' {
        index.push(i);
      }
    }

    Program {
      filename: name,
      text: text,
      tokens: Vec::new(),
      row_index: index,
    }
  }

  pub fn get_location(&self, index: usize) -> (usize, usize) {
    assert!(index < self.text.len(), "Attempted to index outside the bounds of the program text");
    print!("Finding location: \n");
    let mut last_pos = 0;
    for (i, pos) in self.row_index.iter().enumerate() {
      if *pos > index {
        return (i, index - last_pos)
      }
      last_pos = *pos + 1;
    }

    panic!("ICE: Found invalid value in Program::row_index!");
  }

  pub fn find_next(&self, ch: char, mut index: usize) -> Option<usize> {
    while index < self.text.len() {
      if self.text[index] == ch {
        return Some(index)
      }
      index += 1;
    }
    None
  }

  pub fn find_prev(&self, ch: char, mut index: usize) -> Option<usize> {
    while index+1 > 0 {
      if self.text[index-1] == ch {
        return Some(index-1)
      }
      index -= 1;
    }
    None
  }

  pub fn substr(&self, start: usize, end: usize) -> String {
    self.text[start..end].iter().collect()
  }

  pub fn get_line(&self, index: usize) -> String {
    let start = match self.find_prev('\n', index) {
      Some(i) => i+1,
      None => 0,
    };
    let end = match self.find_next('\n', index) {
      Some(i) => i,
      None => self.tokens.len(),
    };

    self.substr(start, end)
  }

  pub fn get_line_with_token(&self, tok: &Token) -> String {
    self.get_line(tok.index)
  }

  pub fn read_while(&self, func: fn(char) -> bool, start: usize) -> String {
    let mut end = start;
    while end < self.text.len() && func(self.text[end]) {
      end += 1;
    }

    self.text[start..end].iter().collect()
  }

  pub fn print_tokens(&self) {
    for token in &self.tokens {
      let string = token.to_string();
      print!("{} ", string);
    }
  }

  pub fn expected_token_error(&self, expected_token: TokenKind, found_token: usize) -> String {
    let line = self.get_line(found_token);
    format!("Expected '{}' but found '{}' instead\n{}", token_kind_to_string(&expected_token), token_kind_to_string(&self.tokens[found_token].kind), line)
  }

  pub fn expect_token(&self, expected_token: TokenKind, pos: usize) -> Result<usize, String> {
    match &self.tokens[pos].kind {
      tok if *tok == expected_token => Ok(pos+1),
      _ => Err(format!("Expected {} token when parsing", token_kind_to_string(&expected_token))),
    }
  }

  pub fn eat_whitespace_tokens(&self, pos: usize) -> Result<usize, String> {
    let mut index = pos;
    while index < self.tokens.len() {
      match &self.tokens[index].kind {
        TokenKind::Newline => (),
        _ => return Ok(index),
      }
      index += 1;
    }
    Err(format!("Reached EOF when parsing:\n{}", self.get_line(pos)))
  }
  
  // TODO Remove
  pub fn check_token(&self, pos: usize, expected_token: TokenKind) -> Result<usize, String> {
    match &self.tokens[pos].kind {
      tok if *tok == expected_token => Ok(pos+1),
      _ => Err(format!("Expected {} token when parsing", token_kind_to_string(&expected_token))),
    }
  }

  pub fn get_token(&self, pos: usize, kind: &TokenKind) -> Result<(Token, usize), String> {
    let mut index = pos;
    while index < self.tokens.len() {
      match &self.tokens[index].kind {
        tok if *tok == *kind => return Ok((self.tokens[index].clone(), index)),
        tok if is_whitespace_token(tok) => (),
        _ => (),//return Err(format!("Expected {} token when parsing", token_kind_to_string(&kind))),
      }
      index += 1;
    }
    Err(format!("Reached EOF when parsing:\n{}", self.get_line(pos)))
  }

  pub fn eat_token(&self, pos: usize, kind: &TokenKind) -> Result<usize, String> {
    match self.get_token(pos, kind) {
      Ok((tok, index)) => Ok(index+1),
      Err(msg) => Err(msg),
    }
  }

  pub fn get_scope(&self, pos: usize, opening: TokenKind) -> Result<(Vec<Token>, usize), String> {
    let mut scope = Vec::new();
    let start_pos = match self.eat_token(pos, &opening) {
      Ok(i) => i,
      Err(msg) => return Err(msg),
    };
    let end_pos = match self.get_token(start_pos, &closing_token(&opening)) {
      Ok((tok, i)) => i,
      Err(msg) => return Err(msg),
    };
    for index in start_pos..end_pos {
      scope.push(self.tokens[index].clone());
    }
    Ok((scope, end_pos+1))
  }
}

pub fn run() {
  let root_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let src_path = root_path.join("src");
  let narrative_path = src_path.join("narrative.txt");
  let mut program = read_program(&narrative_path);
  match lex(&program) {
    Ok(tok) => program.tokens = tok,
    Err(msg) => panic!("Error: {}\n", msg),
  }

  //program.print_tokens();
  match parse(&program) {
    Ok(_) => panic!("OK\n"),
    Err(msg) => panic!("Error: {}\n", msg),
  }
}

// Creates a program by reading in a file from the given path
fn read_program(filename: &Path) -> Program {
  let display = filename.display();
  let mut file = match File::open(&filename) {
    Ok(f) => f,
    Err(msg) => panic!("Could not open {}: {}", display, msg),
  };

  let mut text_string = String::new();
  match file.read_to_string(&mut text_string) {
    Ok(_) => (),
    Err(msg) => panic!("Could not read {}: {}", display, msg),
  };

  let text = text_string.chars().collect();
  Program::new(display.to_string(), text)
}

// Tests whether or not a character is considered to be a "text" character
fn is_text(ch: char) -> bool {
  match ch {
    'a'..='z' => true,
    'A'..='Z' => true,
    '0'..='9' => true,
    '"' => true,
    ',' => true,
    '.' => true,
    '-' => true,
    '_' => true,
    ':' => true,
    ';' => true,
    '!' => true,
    '?' => true,
    '/' => true,
    _ => false,
  }
}

fn is_whitespace(ch: char) -> bool {
  match ch {
    ' ' => true,
    '\t' => true,
    '\n' => true,
    '\r' => true,
    _ => false,
  }
}

fn is_whitespace_token(kind: &TokenKind) -> bool {
  match kind {
    TokenKind::Newline => true,
    _ => false,
  }
}

fn is_keyword(text: &str) -> bool {
  match text {
    "ROOM" => true,
    "BREAK" => true,
    _ => false,
  }
}

fn closing_token(kind: &TokenKind) -> TokenKind {
  match kind {
    TokenKind::Ampersand => TokenKind::Ampersand,
    TokenKind::Asterisk => TokenKind::Asterisk,
    TokenKind::At  => TokenKind::At,
    TokenKind::Caret  => TokenKind::Caret,
    TokenKind::Dollar => TokenKind::Dollar,
    TokenKind::LessThan => TokenKind::GreaterThan,
    TokenKind::OpenCurlyBrace => TokenKind::CloseCurlyBrace,
    TokenKind::OpenParen  => TokenKind::CloseParen,
    TokenKind::OpenSquareBracket  => TokenKind::CloseSquareBracket,
    TokenKind::Percent => TokenKind::Percent,
    TokenKind::Pipe => TokenKind::Pipe,
    TokenKind::Pound => TokenKind::Pound,
    TokenKind::Tilde => TokenKind::Tilde,
    tok => tok.clone(),
  }
}

// Lex the program into an array of tokens
fn lex(program: &Program) -> Result<Vec<Token>, String> {
  let mut index = 0;
  let mut len;
  let mut tokens = Vec::new();

  while index < program.text.len() {
    let ch = program.text[index];
    len = 1;
    
    match ch {
      '#' => tokens.push(Token::new(TokenKind::Pound, index)),
      '%' => tokens.push(Token::new(TokenKind::Percent, index)),
      '&' => tokens.push(Token::new(TokenKind::Ampersand, index)),
      '(' => tokens.push(Token::new(TokenKind::OpenParen, index)),
      ')' => tokens.push(Token::new(TokenKind::CloseParen, index)),
      '*' => tokens.push(Token::new(TokenKind::Asterisk, index)),
      '+' => tokens.push(Token::new(TokenKind::Plus, index)),
      '<' => tokens.push(Token::new(TokenKind::LessThan, index)),
      '>' => tokens.push(Token::new(TokenKind::GreaterThan, index)),
      '@' => tokens.push(Token::new(TokenKind::At, index)),
      '-' => tokens.push(Token::new(TokenKind::Minus, index)),
      '[' => tokens.push(Token::new(TokenKind::OpenSquareBracket, index)),
      ']' => tokens.push(Token::new(TokenKind::CloseSquareBracket, index)),
      '^' => tokens.push(Token::new(TokenKind::Caret, index)),
      '{' => tokens.push(Token::new(TokenKind::OpenCurlyBrace, index)),
      '}' => tokens.push(Token::new(TokenKind::CloseCurlyBrace, index)),
      '|' => tokens.push(Token::new(TokenKind::Pipe, index)),
      '~' => tokens.push(Token::new(TokenKind::Tilde, index)),
      '$' => tokens.push(Token::new(TokenKind::Dollar, index)),
      '\n' => tokens.push(Token::new(TokenKind::Newline, index)),
      '\r' => (),
      ' ' => (),
      '\t' => (),
      // fixme: not proporly handling '?', '"', etc.
      '?' => (),
      '"' => (),
      '”' => (),
      '“' => (),
      '’' => (),
      '\'' => (),
      '…' => (),

      ch if is_text(ch) => {
        let mut text = program.read_while(|ch| {is_text(ch) || ch == ' '}, index);
        match text.pop() {
          Some(ch) => {
            if !is_whitespace(ch) {
              text.push(ch)
            }
          },
          None => (),
        }
        len = text.len();
        if is_keyword(&text) {
          tokens.push(Token::new(TokenKind::Keyword(text), index));
        } else {
          tokens.push(Token::new(TokenKind::Text(text), index));
        }
      },
      other => {
        let (row, col) = program.get_location(index);
        let sol = match program.find_prev('\n', index) {
          Some(i) => i,
          None => 0,
        };
        let eol = match program.find_next('\n', index) {
          Some(i) => i,
          None => program.text.len(),
        };
        let line = program.substr(sol, eol);
        let highlight = format!("{:<1$}^", " ", col);
        let msg = format!("Unknown symbol \'{}\' (ascii: {}) found at {}:{}:{}\n{}\n{}", other, other as u32, program.filename, row+1, col+1, line, highlight); 
        return Err(msg);
      },
    }

    index += len;
  }

  Ok(tokens)
}

// fn eat_whitespace_tokens(program: &Program, pos: usize) -> Result<usize, String> {
//   let mut index = pos;
//   while index < program.tokens.len() {
//     match &program.tokens[index].kind {
//       TokenKind::Newline => (),
//       _ => return Ok(index),
//     }
//     index += 1;
//   }
//   Err(format!("Reached EOF when parsing:\n{}", program.get_line(pos)))
// }
// 
// fn check_token(program: &Program, pos: usize, expected_token: TokenKind) -> Result<usize, String> {
//   match &program.tokens[pos].kind {
//     tok if *tok == expected_token => Ok(pos+1),
//     _ => Err(format!("Expected {} token when parsing", token_kind_to_string(&expected_token))),
//   }
// }

// fn parse_square_brackets(program: &Program, mut pos: usize) -> Result<(ParseNode, usize), String> {
//   
// }

fn parse_room(program: &Program, mut pos: usize) -> Result<(ParseNode, usize), String> {
  let name = match program.check_token(pos, TokenKind::Keyword("ROOM".to_string()))
    .and_then(|i| program.eat_whitespace_tokens(i))
    .and_then(|i| program.get_scope(i, TokenKind::OpenSquareBracket)) {
    Ok((room_name, index)) => {
      pos = index;
      if room_name.len() != 1 {
        return Err(format!("Expected 1 token for room name but found {}", room_name.len()))
      } else {
        room_name[0].clone()
      }
    },
    Err(msg) => return Err(msg),
  };
  Ok((ParseNode::new(), pos))
}

fn parse(program: &Program) -> Result<ParseNode, String> {
  let mut pos = 0;
  let mut node;
  match program.eat_whitespace_tokens(pos) {
    Ok(i) => pos = i,
    Err(msg) => return Err(msg),
  }
  while pos < program.tokens.len() {
    match &program.tokens[pos].kind {
      TokenKind::Keyword(t) if t == "ROOM" => {
        match parse_room(&program, pos) {
          Ok((n, i)) => {
            node = n;
            pos = i;
          },
          Err(msg) => return Err(msg),
        }
      },
      TokenKind::Newline => (),
      _ => return Err(format!("Found incorect token while parsing {} or TODO\n{}", program.tokens[pos].to_string(), program.get_line_with_token(&program.tokens[pos]))),
    }
    pos += 1
  }

  Err("Finished parsing".to_string())
}

#[test]
fn test_token_index() {
  let args: Vec<String> = std::env::args().collect();
  let current_path = Path::new(&args[0]).parent().unwrap();
  let root_path = current_path.join("..").join("..").join("..");
  let src_path = root_path.join("src");
  let narrative_path = src_path.join("narrative.txt");
  let mut program = read_program(&narrative_path);
  match lex(&program) {
    Ok(tok) => program.tokens = tok,
    Err(msg) => assert!(false, "Error: {}\n", msg),
  }

  for i in 0..program.tokens.len() {
    assert!(i == program.tokens[i].index, format!("Invalid token index: Found {} but expected {}.\n\"{}\"", program.tokens[i].index, i, program.get_line(i)));
  }
}

