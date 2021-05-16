

impl From<String> for TokenKind {
  fn from(other: String) -> TokenKind {
    TokenKind::Text(other)
  }
}

impl<'a> From<&'a str> for TokenKind {
  fn from(other: &'a str) -> TokenKind {
    TokenKind::Text(other.to_string())
  }
}

fn main() {}

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
    _ => false,
  }
}

fn while_pred<T>(input: &str, pred: T) -> (&str, usize)
where
  T: Fn (char) -> bool
{
  let mut count = 0;
  for ch in input.chars() {
    if pred(ch) {
      count += ch.len_utf8()
    } else {
      break
    }
  }

  (&input[..count], count)
}

fn tokenize_text(input: &str) -> Result<(TokenKind, usize), &'static str> {
  match input.chars().next() {
    None => return Err("Unexpected EOF when tokenizing text"),
    _ => {},
  }

  let (text, bytes_read) = 
    while_pred(input, |ch| is_text(ch));

  if bytes_read == 0 {
    return Err("No text found")
  } else {
    return Ok((TokenKind::Text(text.to_string()), bytes_read))
  }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum TokenKind {
    Text(String),
    Ampersand,
    Asterisk,
    At, 
    Caret, 
    CloseCurlyBrace,
    CloseParen, 
    CloseSquareBracket, 
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
}
fn tokenize(input: &str) -> Result<Vec<TokenKind>, &'static str> {
  let mut index = 0;
  let mut tokens = Vec::new();
  while index < input.len() {
    let mut nextChar = match input.chars().next() { Some(ch) => ch, None => return Err("EOF") };
    let mut charLen = nextChar.len_utf8();
    match nextChar {
      '#' => tokens.push(TokenKind::Pound),
      '%' => tokens.push(TokenKind::Percent),
      '&' => tokens.push(TokenKind::Ampersand),
      '(' => tokens.push(TokenKind::OpenParen),
      ')' => tokens.push(TokenKind::CloseParen),
      '*' => tokens.push(TokenKind::Asterisk),
      '+' => tokens.push(TokenKind::Plus),
      '<' => tokens.push(TokenKind::LessThan),
      '>' => tokens.push(TokenKind::GreaterThan),
      '@' => tokens.push(TokenKind::At),
      '-' => tokens.push(TokenKind::Minus),
      '[' => tokens.push(TokenKind::OpenSquareBracket),
      ']' => tokens.push(TokenKind::CloseSquareBracket),
      '^' => tokens.push(TokenKind::Caret),
      '{' => tokens.push(TokenKind::OpenCurlyBrace),
      '}' => tokens.push(TokenKind::CloseCurlyBrace),
      '|' => tokens.push(TokenKind::Pipe),
      '~' => tokens.push(TokenKind::Tilde),
      ch if ch.is_ascii() && ch as u8 > 31 => match tokenize_text(&input[index..]) {
        Ok((tok, bytes)) => { 
          tokens.push(tok);
          charLen = bytes;
        },
        Err(string) => return Err(string),
      },
      _ => return Err("Unknown symbol found"),
    }

    index = charLen
  }

  Ok(tokens)
}


#[test]
fn tokenize_text_test() {
  let hello = "Hello!";
  let world = "&world";
  assert_eq!(Ok((TokenKind::Text("Hello".to_string()), 5)), tokenize_text(hello));
  assert_eq!(Err("No text found"), tokenize_text(world));
}
