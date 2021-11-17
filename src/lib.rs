extern crate wasm_bindgen;
extern crate cfg_if;

use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::fmt::{self, Display};
use std::collections::HashMap;
use std::collections::HashSet;
use std::{thread, time};
use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

// A collection of utility functions
mod util {
  use std::fmt::{self, Display};

  #[derive(Clone, Copy)]
  pub struct DisplayRepeat<T>(usize, T);
  
  impl<T: Display> Display for DisplayRepeat<T> {
      fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
          for _ in 0..self.0 {
              self.1.fmt(f)?;
          }
          Ok(())
      }
  }
  
  pub fn repeat<T>(times: usize, item: T) -> DisplayRepeat<T> {
      DisplayRepeat(times, item)
  }
}

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

#[derive(Debug, Clone)]
pub enum InventoryKind {
  Personal,
  Room,
  Global,
}

#[derive(Debug, Clone)]
pub enum InventoryAction {
  Add,
  Remove,
  Check,
}

#[derive(Debug, Clone)]
pub struct InventoryModification {
  pub inventory: InventoryKind,
  pub action: InventoryAction,
  pub item: Token,
}

#[derive(Debug, Clone)]
pub struct GameText {
  pub text: Vec<Token>,
  pub itallic: bool,
  pub bold: bool,
  pub color: u32,
}

#[derive(Debug, Clone)]
pub struct GameAudio {
  pub path: Token,
  pub sound_effect: bool,
}

#[derive(Debug, Clone)]
pub struct GameItem {
  pub name: Token,
  pub action: InventoryAction,
  pub inventory: InventoryKind,
}

#[derive(Debug, Clone)]
pub struct GameAction {
  pub action: Token,
  pub name: Token,
  pub requirements: Vec<GameItem>,
  pub scope: Vec<ParseNode>,
}

#[derive(Debug, Clone)]
pub struct GameRoom {
  pub name: Token,
  pub requirements: Vec<GameItem>,
  pub scope: Vec<ParseNode>,
}


// The more complex grammar expressions
#[derive(Debug, Clone)]
pub enum Expr {
  Break,
  Delay(Token),
  Room(GameRoom),
  Goto(Token),
  Text(GameText),
  Audio(GameAudio),
  Action(GameAction),
  Require(GameItem),
  Modify(GameItem),
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

fn expr_to_string(expr: &Expr) -> String {
  match expr {
    Expr::Break => format!("|BREAK|"),
    Expr::Delay(token) => format!("{}", token_kind_to_string(&token.kind)),
    Expr::Room(gameRoom) => format!("Room |{}|", gameRoom.name.to_string()),
    Expr::Goto(token) => format!("[[{}]]", token_kind_to_string(&token.kind)),
    Expr::Text(gameText) => format!("{}", (&gameText.text).into_iter().map(|t| -> String { t.to_string() }).collect::<String>()),
    Expr::Audio(gameAudio) => format!("<{}>", gameAudio.path.to_string()),
    Expr::Action(gameAction) => format!("{} |{}|", gameAction.action.to_string(), gameAction.name.to_string()),
    Expr::Require(gameItem) => format!("REQUIRE({})", gameItem.name.to_string()),
    Expr::Modify(gameItem) => format!("MODIFY({})", gameItem.name.to_string()),
  }
}

fn token_kind_to_string(kind: &TokenKind) -> String {
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
    _ => format!("TODO"),
  }
}

fn match_token_kind(kind_a: &TokenKind, kind_b: &TokenKind) -> bool {
  match (kind_a, kind_b) {
    (TokenKind::Text(t_a), TokenKind::Text(t_b)) => if t_a == t_b { return true } else { return false },
    (TokenKind::Keyword(t_a), TokenKind::Keyword(t_b)) => if t_a == t_b { return true } else { return false },
    _ => if kind_a == kind_b { return true } else { return false },
  }
}

pub struct Inventory {
  pub personal: HashSet<String>,
  pub room: HashMap<String, HashSet<String>>,
  pub global: HashSet<String>,
}

impl Inventory {
  pub fn new() -> Inventory {
    Inventory {
      personal: HashSet::new(),
      room: HashMap::new(),
      global: HashSet::new(),
    }
  }

  pub fn to_string(&self, room: &String) -> String {
    let personal = self.personal.iter().fold(String::from("Personal: ["), |a, b| a + " " + b) + " ]";
    let room = match self.room.get(room) {
      Some(r) => r.iter().fold(String::from("Room: ["), |a, b| a + " " + b) + " ]",
      None => String::from("Room: [ ]"),
    };
    let global = self.global.iter().fold(String::from("Global: ["), |a, b| a + " " + b) + " ]";

    format!("{}\n{}\n{}\n", personal, room, global)
  }

  // Check is the item is in the inventory.
  pub fn check_item(&self, item: &GameItem, room: &String) -> bool {
    match &item.inventory {
      InventoryKind::Personal => { return self.personal.contains(&item.name.to_string()) },
      InventoryKind::Global => { return self.global.contains(&item.name.to_string()) },
      InventoryKind::Room => {
        match self.room.get(room) {
          Some(room_inventory) => room_inventory.contains(&item.name.to_string()),
          None => false,
        }
      },
    }
  }

  // Must have all items for check to pass.
  pub fn check_items(&self, items: &Vec<GameItem>, room: &String) -> bool {
    for i in 0..items.len() {
      if !self.check_item(&items[i], &room) {
        return false;
      }
    }
    true
  }

  pub fn add_item(&mut self, item: &GameItem, room: &String) {
    match item.inventory {
      InventoryKind::Personal => { self.personal.insert(item.name.to_string()); },
      InventoryKind::Global => { self.global.insert(item.name.to_string()); },
      InventoryKind::Room => {
        match self.room.get_mut(room) {
          Some(room_inventory) => { room_inventory.insert(item.name.to_string()); },
          None => {
            let mut new_inventory = HashSet::new();
            new_inventory.insert(item.name.to_string());
            self.room.insert(room.to_string(), new_inventory);
          },
        }
      },
    }
  }

  pub fn remove_item(&mut self, item: &GameItem, room: &String) {
    match item.inventory {
      InventoryKind::Personal => { self.personal.remove(&item.name.to_string()); },
      InventoryKind::Global => { self.global.remove(&item.name.to_string()); },
      InventoryKind::Room => {
        match self.room.get_mut(room) {
          Some(room_inventory) => { room_inventory.remove(&item.name.to_string()); },
          None => (),
        }
      }
    }
  }

  pub fn modify(&mut self, item: &GameItem, room: &String) {
    match item.action {
      InventoryAction::Add => self.add_item(&item, &room),
      InventoryAction::Remove => self.remove_item(&item, &room),
      InventoryAction::Check => panic!("ICE: Attempting to modify a check item '{}'", item.name.to_string()),
    }
  }
}

#[wasm_bindgen]
pub struct Game {
  inventory: Inventory,
  rooms: HashMap<String, (Vec<GameRoom>, Vec<GameAction>)>,
  current_room_name: String,
  current_room_index: usize,
}

impl Game {
  pub fn new(rooms_map: &HashMap<String, (Vec<GameRoom>, Vec<GameAction>)>) -> Game {
    let inventory = Inventory::new();
    let mut new_map = HashMap::new();
    for (key, val) in rooms_map.iter() {
      let (r, a) = val;
      new_map.insert(key.to_string(), (r.clone(), a.clone()));
    }

    Game {
      inventory: inventory,
      rooms: new_map,
      current_room_name: "init".to_string(),
      current_room_index: 0,
    }
  }

  pub fn print_room(&self, room_name: &String) -> String {
    let room = match self.find_room(&room_name) {
      Ok((r, a)) => r,
      Err(msg) => panic!(msg),
    };

    let mut output = "{\n  \"room_output\": [\n".to_string();
    for i in 0..room.scope.len() {
      match &room.scope[i].value {
        Expr::Break => { 
          output.push_str("    \"|BREAK|\",\n");
        },
        Expr::Delay(token) => { output.push_str(&format!("    \"|{}|\",\n", token.to_string())); },  // TODO: actually read delay.
        Expr::Room(game_room) => { panic!("Discovered Room '{}' inside of Room '{}'", game_room.name.to_string(), self.current_room_name); },
        Expr::Goto(token) => {
          output.push_str(&format!("    \"[[{}]]\",\n", token.to_string()));
          //let new_room_name = token.to_string();
          //match find_room(&rooms, &new_room_name, &inventory) {
          //  Ok((r, a)) => {
          //    room = r;
          //    actions = a;
          //  },
          //  Err(msg) => return Err(msg),
          //};
          //skip_actions = true;
          //break;
        },
        Expr::Text(game_text) => {
          output.push_str(&format!("    \"{}\",\n", format!("{} ", (&game_text.text).into_iter().map(|t| -> String { t.to_string() }).collect::<String>())));
        },
        Expr::Audio(game_audio) => { output.push_str(&format!("    \"<{}>\",\n", game_audio.path.to_string())); },
        Expr::Action(game_action) => { panic!("Discovered Action '{} |{}|' inside of Room '{}'", game_action.action.to_string(), game_action.name.to_string(), self.current_room_name); },
        Expr::Require(game_item) => { panic!("Discovered 'Require({})', inside of Room '{}'", game_item.name.to_string(), self.current_room_name); },
        Expr::Modify(game_item) => { 
          //self.inventory.modify(&game_item, &self.current_room_name); 
          output.push_str(&format!("    \"^{}^\",\n", game_item.name.to_string()));
        },
      }
    }
    output.push_str("  ],\n}");

    output
  }

  pub fn find_room(&self, room_name: &String) -> Result<(GameRoom, Vec<GameAction>), String> {
    match self.rooms.get(room_name) {
      Some((game_rooms, game_actions)) => {
        for i in 0..game_rooms.len() {
          if self.inventory.check_items(&game_rooms[i].requirements, &room_name) {
            return Ok((game_rooms[i].clone(), game_actions.to_vec()));  // TODO handle lifetime so that refrences can be returned.
          }
        }
        return Err(format!("No acceptable room could be found for '{}'", room_name));
      },
      None => {
        println!("Room keys");
        for key in self.rooms.keys() {
          println!("{}", key);
        }
        return Err(format!("Could not find room '{}'", room_name))
      },
    }
  }

  pub fn find_action_index(&self, action_type: &String, action_name: &String) -> Result<usize, String> {
    let actions = match self.rooms.get(&self.current_room_name) {
      Some((r, a)) => a,
      None => return Err(format!("ICE: Could not find the room '{}'", self.current_room_name)),
    };

    for i in 0..actions.len() {
      if actions[i].action.to_string().to_uppercase() == action_type.to_string().to_uppercase() && actions[i].name.to_string().to_lowercase() == action_name.to_string().to_lowercase() {
        if self.inventory.check_items(&actions[i].requirements, &self.current_room_name) {
          return Ok(i);
        }
      }
    }
    
    return Err(format!("Invalid command '{} {}', try again", action_type, action_name));
  }

  pub fn print_scope(&mut self, scope: &Vec<ParseNode>) -> String {
    let mut output = String::new();
    for i in 0..scope.len() {
      match &scope[i].value {
        Expr::Break => { 
          output.push_str("|BREAK|\n");
        },
        Expr::Delay(token) => { output.push_str(&format!("|{}|\n", token.to_string())); },  // TODO: actually read delay.
        Expr::Room(game_room) => { panic!("Discovered Room '{}' inside of Room '{}'", game_room.name.to_string(), self.current_room_name); },
        Expr::Goto(token) => {
          //output.push_str(&format!("[[{}]]\n", token.to_string()));
          let new_room_name = token.to_string();
          match self.find_room(&new_room_name) {
            Ok((r, a)) => {
              self.current_room_name = r.name.to_string();
            },
            Err(msg) => return format!("Error: {}", msg),
          };
          //skip_actions = true;
          //break;
        },
        Expr::Text(game_text) => {
          if i+1 < scope.len() {
            match &scope[i+1].value {
              Expr::Text(t) => output.push_str(&format!("{} ", (&game_text.text).into_iter().map(|t| -> String { t.to_string() }).collect::<String>())),
              _ => output.push_str(&format!("{}\n", format!("{} ", (&game_text.text).into_iter().map(|t| -> String { t.to_string() }).collect::<String>()))),
            }
          } else {
            output.push_str(&format!("{}\n", format!("{} ", (&game_text.text).into_iter().map(|t| -> String { t.to_string() }).collect::<String>())));
          }
        },
        Expr::Audio(game_audio) => { output.push_str(&format!("<{}>\n", game_audio.path.to_string())); },
        Expr::Action(game_action) => { panic!("Discovered Action '{} |{}|' inside of Room '{}'", game_action.action.to_string(), game_action.name.to_string(), self.current_room_name); },
        Expr::Require(game_item) => { panic!("Discovered 'Require({})', inside of Room '{}'", game_item.name.to_string(), self.current_room_name); },
        Expr::Modify(game_item) => { 
          self.inventory.modify(&game_item, &self.current_room_name); 
        },
      }
    }

    output
  }

  pub fn get_current_room(&self) -> GameRoom {
    self.rooms.get(&self.current_room_name).unwrap().0[self.current_room_index].clone()
  }
}

#[wasm_bindgen]
impl Game {
  pub fn start(&mut self) -> String {
     let rooms = match self.rooms.get(&self.current_room_name) {
       Some((r, a)) => r,
       None => return format!("Error: ROOM |{}| not found", self.current_room_name),
     };

    for i in 0..rooms.len() {
      if self.inventory.check_items(&rooms[i].requirements, &self.current_room_name) {
        return self.print_scope(&rooms[i].scope.clone());
      }
    }

    format!("Error: Could not find any ROOM |{}| for which satisfied the current inventory requirements\n{}", self.current_room_name, self.inventory.to_string(&self.current_room_name))
  }

  pub fn list_all_rooms(&self) -> String {
    self.rooms.keys().fold(String::new(), |a, b| a + b + "\n")
  }

  pub fn print_current_room(&mut self) -> String {
    self.print_scope(&self.get_current_room().scope)   
  }

  pub fn query(&mut self, action: String, command: String) -> String {
    let index = match self.find_action_index(&action, &command) {
      Ok(i) => i,
      Err(msg) => return format!("Could not find action: {} |{}| under ROOM |{}|. ({})", action, command, self.current_room_name, msg),
    };

    let scope = self.rooms.get(&self.current_room_name).unwrap().1[index].scope.clone();

    self.print_scope(&scope)
  }

  pub fn print_inventory(&self) -> String {
    self.inventory.to_string(&self.current_room_name).to_string()
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
  pub fn new(expr: Expr) -> ParseNode {
    ParseNode {
      children: Vec::new(),
      value: expr,
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

  pub fn find_next_token(&self, kind: TokenKind, mut start_pos: usize) -> Option<usize> {
    while start_pos < self.tokens.len() {
      if self.tokens[start_pos].kind == kind {
        return Some(start_pos)
      }
      start_pos += 1;
    }
    None
  }

  pub fn find_prev_token(&self, kind: TokenKind, mut start_pos: usize) -> Option<usize> {
    while start_pos+1 > 0 {
      if self.tokens[start_pos-1].kind == kind {
        return Some(start_pos-1)
      }
      start_pos -= 1;
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

  pub fn get_location_with_token(&self, tok: &Token) -> (usize, usize) {
    self.get_location(tok.index)
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

  pub fn peek_token(&self, pos: usize, token: TokenKind) -> bool {
    if pos+1 < self.tokens.len() && match_token_kind(&token, &self.tokens[pos+1].kind) {
      true
    } else {
      false
    }
  }
}

pub fn run() {
  let root_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let src_path = root_path.join("src");
  let narrative_path = src_path.join("narrative.txt");
  let mut program = read_program(&narrative_path);
  //let mut program = read_program_from_string(&GAME_TEXT.to_string());
  match lex(&program) {
    Ok(tok) => program.tokens = tok,
    Err(msg) => panic!("Error: {}\n", msg),
  }

  //program.print_tokens();
  let nodes = match parse(&program) {
    Ok(nodes) => nodes,
    Err(msg) => panic!("Error: {}\n", msg),
  };

  let rooms = match setup_rooms(&nodes) {
    Ok(rooms) => rooms,
    Err(msg) => panic!("Error: {}\n", msg),
  };

  match start_game(&rooms) {
    Ok(_) => (),
    Err(msg) => panic!("Error: {}\n", msg),
  }
}

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
  a + b
}

#[wasm_bindgen]
pub fn compile(text: String) -> Game {
  let mut program = read_program_from_string(&text);
  match lex(&program) {
    Ok(tok) => program.tokens = tok,
    Err(msg) => panic!("Error: {}\n", msg),
  }


  //program.print_tokens();
  let nodes = match parse(&program) {
    Ok(nodes) => nodes,
    Err(msg) => panic!("Error: {}\n", msg),
  };

  let rooms = match setup_rooms(&nodes) {
    Ok(rooms) => rooms,
    Err(msg) => panic!("Error: {}\n", msg),
  };

  let game = Game::new(&rooms);
  game
}

fn find_room(rooms: &HashMap<String, (Vec<GameRoom>, Vec<GameAction>)>, room_name: &String, inventory: &Inventory) -> Result<(GameRoom, Vec<GameAction>), String> {
  match rooms.get(room_name) {
    Some((game_rooms, game_actions)) => {
      for i in 0..game_rooms.len() {
        if inventory.check_items(&game_rooms[i].requirements, &room_name) {
          return Ok((game_rooms[i].clone(), game_actions.to_vec()));  // TODO handle lifetime so that refrences can be returned.
        }
      }
      return Err(format!("No acceptable room could be found for '{}'", room_name));
    },
    None => {
      println!("Room keys");
      for key in rooms.keys() {
        println!("{}", key);
      }
      return Err(format!("Could not find room '{}'", room_name))
    },
  }
}

fn find_action(actions: &Vec<GameAction>, room_name: &String, action_type: String, action_name: String, inventory: &Inventory) -> Result<usize, String> {
  for i in 0..actions.len() {
    if actions[i].action.to_string().to_uppercase() == action_type.to_string().to_uppercase() && actions[i].name.to_string().to_lowercase() == action_name.to_string().to_lowercase() {
      if inventory.check_items(&actions[i].requirements, &room_name) {
        return Ok(i);
      }
    }
  }
  
  return Err(format!("Invalid command '{} {}', try again", action_type, action_name));
}

fn start_game(rooms: &HashMap<String, (Vec<GameRoom>, Vec<GameAction>)>) -> Result<bool, String> {
  let mut inventory = Inventory::new();
  let mut current_room = String::from("init");
  let (mut room, mut actions) = match find_room(&rooms, &current_room, &inventory) {
    Ok((room, actions)) => (room, actions),
    Err(msg) => return Err(msg),
  };

  let mut line = String::new();
  while true {
    println!("Now in Room: {}", room.name.to_string());
    let mut skip_actions = false;
    for i in 0..room.scope.len() {
      match &room.scope[i].value {
        Expr::Break => { 
          print!("\n");
          std::io::stdin().read_line(&mut line); 
          line.clear();
        },
        Expr::Delay(token) => { thread::sleep(time::Duration::from_secs(5)); },  // TODO: actually read delay.
        Expr::Room(game_room) => { return Err(format!("Discovered Room '{}' inside of Room '{}'", game_room.name.to_string(), current_room)); },
        Expr::Goto(token) => {
          let new_room_name = token.to_string();
          match find_room(&rooms, &new_room_name, &inventory) {
            Ok((r, a)) => {
              room = r;
              actions = a;
            },
            Err(msg) => return Err(msg),
          };
          skip_actions = true;
          break;
        },
        Expr::Text(game_text) => {
          print!("{} ", (&game_text.text).into_iter().map(|t| -> String { t.to_string() }).collect::<String>());
        },
        Expr::Audio(game_audio) => { continue; },
        Expr::Action(game_action) => { return Err(format!("Discovered Action '{} |{}|' inside of Room '{}'", game_action.action.to_string(), game_action.name.to_string(), current_room)); },
        Expr::Require(game_item) => { return Err(format!("Discovered 'Require({})', inside of Room '{}'", game_item.name.to_string(), current_room)); },
        Expr::Modify(game_item) => { inventory.modify(&game_item, &current_room); },
      }
    }

    if skip_actions {
      continue;
    }


    let mut leave_actions = false;
    while !leave_actions {

      print!("\n");
      let mut index: usize = 0;
      while true {
        let mut command: String = "MISC".to_string();
        let mut argument: String = "".to_string();
        while true {
          line.clear();
          std::io::stdin().read_line(&mut line).expect("Failed to read input");
          line = format!("{}", line.replace('\n', ""));
          line = format!("{}", line.replace('\r', ""));
          let split: Vec<&str> = line.split(" ").collect();
          if split.len() == 0 {
            continue;
          } else if split.len() == 1 {
            argument = split[0].to_lowercase();
            break;
          } else {
            command = split[0].to_uppercase();
            argument = split[1].to_lowercase();
            break;
          }
        }

        match find_action(&actions, &current_room, command.clone(), argument.clone(), &inventory) {
          Ok(i) => {
            index = i;
            break;
          },
          Err(msg) => println!("{}", msg),
        }
      }
      let action = &actions[index];

      for i in 0..action.scope.len() {
        match &action.scope[i].value {
          Expr::Break => { 
            print!("\n");
            std::io::stdin().read_line(&mut line); 
            line.clear();
          },
          Expr::Delay(token) => { thread::sleep(time::Duration::from_secs(5)); },  // TODO: actually read delay.
          Expr::Room(game_room) => { return Err(format!("Discovered Room '{}' inside of Action '{} |{}|'", game_room.name.to_string(), action.action.to_string(), action.name.to_string())); },
          Expr::Goto(token) => {
            let new_room_name = token.to_string();
            match find_room(&rooms, &new_room_name, &inventory) {
              Ok((r, a)) => {
                room = r;
                actions = a;
              },
              Err(msg) => return Err(msg),
            };
            leave_actions = true;
            break;
          },
          Expr::Text(game_text) => {
            print!("{} ", (&game_text.text).into_iter().map(|t| -> String { t.to_string() }).collect::<String>());
          },
          Expr::Audio(game_audio) => { continue; },
          Expr::Action(game_action) => { return Err(format!("Discovered Action '{} |{}|' inside of Action '{} |{}|'", game_action.action.to_string(), game_action.name.to_string(), action.action.to_string(), action.name.to_string())); },
          Expr::Require(game_item) => { return Err(format!("Discovered 'Require({})', inside of Action '{} |{}|'", game_item.name.to_string(), action.action.to_string(), action.name.to_string())); },
          Expr::Modify(game_item) => { inventory.modify(&game_item, &current_room); },
        }
      }
    }
  }
  
  
  Ok(true)
}

fn setup_rooms(nodes: &Vec<ParseNode>) -> Result<HashMap<String, (Vec<GameRoom>, Vec<GameAction>)>, String> {
  let mut rooms = HashMap::<String, (Vec<GameRoom>, Vec<GameAction>)>::new();
  let mut grouped_rooms = Vec::<GameRoom>::new();
  let mut grouped_actions = Vec::<GameAction>::new();
  let mut current_room_name: String = "".to_string();
  for i in 0..nodes.len() {
    match &nodes[i].value {
      Expr::Room(gameRoom) => {
        if gameRoom.name.to_string() != current_room_name {
          if !grouped_rooms.is_empty() {
            if rooms.contains_key(&current_room_name) {
              return Err(format!("The room name '{}' was found in multiple differnt sections", current_room_name));
            }
          }

          rooms.insert(current_room_name.clone(), (grouped_rooms.clone(), grouped_actions.clone()));
          grouped_rooms.clear();
          grouped_actions.clear();
          current_room_name = gameRoom.name.to_string();
        }
        grouped_rooms.push(gameRoom.clone());
      },
      Expr::Action(gameAction) => {
        grouped_actions.push(gameAction.clone());
      },
      other => return Err(format!("Found '{}' on the top level tree nodes", expr_to_string(&other))),
    }
  }

  if !grouped_rooms.is_empty() {
    if rooms.contains_key(&current_room_name) {
      return Err(format!("The room name '{}' was found in multiple differnt sections", current_room_name));
    }
    rooms.insert(current_room_name.clone(),(grouped_rooms.clone(), grouped_actions.clone()));
  }

  Ok(rooms)
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
  println!("DISPLAY: {}", display.to_string());
  Program::new(display.to_string(), text)
}

fn read_program_from_string(text: &String) -> Program {
  //let display = filename.display();
  //let mut file = match File::open(&filename) {
  //  Ok(f) => f,
  //  Err(msg) => panic!("Could not open {}: {}", display, msg),
  //};

  //let mut text_string = String::new();
  //match file.read_to_string(&mut text_string) {
  //  Ok(_) => (),
  //  Err(msg) => panic!("Could not read {}: {}", display, msg),
  //};

  //let text = text_string.chars().collect();
  let text_string = text.to_string();
  let text = text_string.chars().collect();
  Program::new("".to_string(), text)
}

// Tests whether or not a character is considered to be a "text" character
fn is_text(ch: char) -> bool {
  match ch {
    'a'..='z' => true,
    'A'..='Z' => true,
    '0'..='9' => true,
    '"' => true,
    '\'' => true,
    ',' => true,
    '.' => true,
    '-' => true,
    '_' => true,
    ':' => true,
    ';' => true,
    '!' => true,
    '?' => true,
    '/' => true,
    '#' => true,
    '@' => true,
    '(' => true,
    ')' => true,
    '~' => true,
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
    "HELP" => true,
    "MISC" => true,
    "EXAMINE" => true,
    "USE" => true,
    "TAKE" => true,
    "TALK" => true,
    "GO" => true,
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

fn int_to_inventory_kind(i: u32) -> InventoryKind {
  match i {
    1 => InventoryKind::Personal,
    2 => InventoryKind::Room,
    3 => InventoryKind::Global,
    _ => panic!("ICE: Attempted to convert {} to InventoryKind", i),
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
      //'#' => tokens.push(Token::new(TokenKind::Pound, index)),
      '%' => tokens.push(Token::new(TokenKind::Percent, index)),
      '&' => tokens.push(Token::new(TokenKind::Ampersand, index)),
      //'(' => tokens.push(Token::new(TokenKind::OpenParen, index)),
      //')' => tokens.push(Token::new(TokenKind::CloseParen, index)),
      '*' => tokens.push(Token::new(TokenKind::Asterisk, index)),
      '+' => tokens.push(Token::new(TokenKind::Plus, index)),
      '<' => tokens.push(Token::new(TokenKind::LessThan, index)),
      '>' => tokens.push(Token::new(TokenKind::GreaterThan, index)),
      //'@' => tokens.push(Token::new(TokenKind::At, index)),
      '-' => tokens.push(Token::new(TokenKind::Minus, index)),
      '[' => tokens.push(Token::new(TokenKind::OpenSquareBracket, index)),
      ']' => tokens.push(Token::new(TokenKind::CloseSquareBracket, index)),
      '^' => tokens.push(Token::new(TokenKind::Caret, index)),
      '{' => tokens.push(Token::new(TokenKind::OpenCurlyBrace, index)),
      '}' => tokens.push(Token::new(TokenKind::CloseCurlyBrace, index)),
      '|' => tokens.push(Token::new(TokenKind::Pipe, index)),
      //'~' => tokens.push(Token::new(TokenKind::Tilde, index)),
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

fn parse_token(program: &Program, pos: usize) -> Result<(ParseNode, usize), String> {
  program.eat_whitespace_tokens(pos).and_then(|i| match &program.tokens[i].kind {
    TokenKind::Text(t) => {
      let mut text = Vec::new();
      text.push(program.tokens[i].clone());
      let game_text = GameText {
        text: text,
        itallic: false,
        bold: false,
        color: 0,
      };
      Ok((ParseNode::new(Expr::Text(game_text)), i+1))
    },

    TokenKind::Keyword(t) => {
      if t == "BREAK" {
        Ok((ParseNode::new(Expr::Break), i))
      } else {
        Err(format!("ICE: Parser does not recognize {} as a keyword", t))
      }
    },
    
    TokenKind::Ampersand => {
      match program.get_scope(i, TokenKind::Ampersand) {
        Ok((tokens, index)) => {
          if tokens.len() != 1 {
            return Err(format!("Inventory requirements must be in the format '&var&', but found '&{}&' instead", tokens.into_iter().map(|t| -> String { t.to_string() }).collect::<String>()))
          }
          let var = match &tokens[0].kind {
            TokenKind::Text(t) => tokens[0].clone(),
            other => return Err("Incorect type in iventory requirement, must be in the format '&var&'".to_string()),
          };
          // TODO Check if correct
          let inventory = InventoryKind::Room;
          let action = InventoryAction::Check;
          let item = GameItem {
            name: var,
            action: action,
            inventory: inventory,
          };
          Ok((ParseNode::new(Expr::Require(item)), index))
        },
        Err(msg) => Err(msg),
      }
    },

    TokenKind::Asterisk => {
      match program.get_scope(i, TokenKind::Asterisk) {
        Ok((tokens, index)) => {
          if tokens.len() != 1 {
            return Err("Sound effect must be in the format '*path/to/audio*'".to_string())
          }
          if let TokenKind::Text(t) = &tokens[0].kind {
            let path = tokens[0].clone();
            // TODO Handle music path somehow
            let music = GameAudio {
              path: path,
              sound_effect: true,
            };
            return Ok((ParseNode::new(Expr::Audio(music)), index))
          } else {
            return Err("Incorrect type in play music action, must be in the format '*path/to/audio*'".to_string())
          }
        },
        Err(msg) => Err(msg),
      }
    },

    TokenKind::At => {
      match program.get_scope(i, TokenKind::At) {
        Ok((tokens, index)) => {
          if tokens.len() == 0 {
            return Err("Colored text must be in the format '@xxxxxx ..@' where xxxxxx is a 24 bit hex number of the color".to_string())
          } 
          // TODO: Check color
          let text_tokens = &tokens[1..];

          for tok in text_tokens.iter() {
            match &tok.kind {
              TokenKind::Text(t) => (),
              other => return Err(format!("Colored text '@xxxxxx ..@' only supports string objects however {} was found", token_kind_to_string(&other))),
            }
          }
          let game_text = GameText {
            text: text_tokens.to_vec(),
            itallic: false,
            bold: false,
            color: 1, // TODO: set color
          };
          Ok((ParseNode::new(Expr::Text(game_text)), index))
        },
        Err(msg) => Err(msg),
      }
    }, 
    TokenKind::Caret => {
      match program.get_scope(i, TokenKind::Caret) {
        Ok((tokens, index)) => {
          if tokens.len() < 2 {
            return Err("Inventory modification must be in the format '^([+-]+)(var)^'".to_string())
          }
          let var = match &tokens[tokens.len()-1].kind {
            TokenKind::Text(t) => tokens[tokens.len()-1].clone(),
            other => return Err("Missing variable in inventory modification, must be in the format '^([+-]+)(var)^'".to_string()),
          };

          let mut pos = 0;
          let mut neg = 0;
          for tok in tokens[..tokens.len()-1].iter() {
            match &tok.kind {
              TokenKind::Plus => {
                pos += 1;
                if neg > 0 {
                  return Err("Inventory modification cannot have both '+' and '-'".to_string())
                }
                if pos > 3 {
                  return Err("Cannot have more than three '+'s in inventory modification".to_string())
                }
              },
              TokenKind::Minus => {
                neg += 1;
                if pos > 0 {
                  return Err("Inventory modification cannot have both '+' and '-'".to_string())
                }
                if neg > 3 {
                  return Err("Cannot have more than three '+'s in inventory modification".to_string())
                }
              },
              other => return Err(format!("Unexpected {} in inventory modification '^..^'", token_kind_to_string(&other))),
            }
          }
          let inventory = match pos {
            0 => int_to_inventory_kind(neg),
            _ => int_to_inventory_kind(pos),
          };
          let action = match neg > 0 {
            true => InventoryAction::Remove,
            false => InventoryAction::Add,
          };
          let item = GameItem {
            name: var,
            action: action,
            inventory: inventory,
          };
          Ok((ParseNode::new(Expr::Modify(item)), index))
        },
        Err(msg) => Err(msg),
      }
    }, 
    TokenKind::CloseCurlyBrace => Err("Found an unexpected '}'".to_string()),
    TokenKind::CloseParen => Err("Found an unexpected ')'".to_string()),
    TokenKind::CloseSquareBracket => Err("Found an unexpected ']'".to_string()), 
    TokenKind::Dollar => {
      match program.get_scope(i, TokenKind::Dollar) {
        Ok((tokens, index)) => {
          if tokens.len() != 1 {
            return Err("Inventory requirements must be in the format '$var$'".to_string())
          }
          let var = match &tokens[0].kind {
            TokenKind::Text(t) => tokens[0].clone(),
            other => return Err("Incorect type in iventory requirement, must be in the format '$var$'".to_string()),
          };
          // TODO Check if correct
          let inventory = InventoryKind::Personal;
          let action = InventoryAction::Check;
          let item = GameItem {
            name: var,
            action: action,
            inventory: inventory,
          };
          Ok((ParseNode::new(Expr::Require(item)), index))
        },
        Err(msg) => Err(msg),
      }
    },
    TokenKind::LessThan => {
      match program.get_scope(i, TokenKind::LessThan) {
        Ok((tokens, index)) => {
          if tokens.len() != 1 {
            return Err(format!("Music must be in the format '<path/to/audio>', found '<{}>' instead", tokens.into_iter().map(|t| -> String { t.to_string() }).collect::<String>()))
          }
          if let TokenKind::Text(t) = &tokens[0].kind {
            let path = tokens[0].clone();
            // TODO Handle music path somehow
            let music = GameAudio {
              path: path,
              sound_effect: false,
            };

            return Ok((ParseNode::new(Expr::Audio(music)), index))
          } else {
            return Err("Incorrect type in play music action, must be in the format '<path/to/audio>'".to_string())
          }
        },
        Err(msg) => Err(msg),
      }
    },

    TokenKind::GreaterThan => Err("Found an unexpected '>'".to_string()),
    TokenKind::Minus => Err("Found an unexpected '-'".to_string()), 
    TokenKind::OpenCurlyBrace => Err("Found an unexpected '{'".to_string()),
    TokenKind::OpenParen => {
      match program.get_scope(i, TokenKind::OpenParen) {
        Ok((tokens, index)) => {
          let mut text = Vec::new();
          //let text = tokens.into_iter().map(|t| -> String { t.to_string() }).collect::<String>();
          text.push(program.tokens[i].clone());
          for token in &tokens {
            text.push(program.tokens[i].clone());
          }
          text.push(program.tokens[index-1].clone());
          let game_text = GameText {
            text: text,
            itallic: false,
            bold: false,
            color: 0,
          };
          Ok((ParseNode::new(Expr::Text(game_text)), index))
        },
        Err(msg) => return Err(msg),
      }
    }, 
    TokenKind::OpenSquareBracket => {
      // TODO: Make better
      if program.tokens[i+1].kind != TokenKind::OpenSquareBracket {
        return Err("TODO: Unhandled label, ie [...], found".to_string())
      }

      match program.get_scope(i+1, TokenKind::OpenSquareBracket) {
        Ok((tokens, index)) => {
          if program.tokens[index].kind != TokenKind::CloseSquareBracket {
            return Err("Missing second ']' in goto statement, ie [[...]]".to_string())
          }
          if tokens.len() != 1 {
            return Err(format!("Goto must be in the format [[some_label]], not [[{}]]", tokens.into_iter().map(|t| -> String { t.to_string() }).collect::<String>()))
          }
          Ok((ParseNode::new(Expr::Goto(tokens[0].clone())), index+1))
        },
        Err(msg) => return Err(msg),
      }
    }, 
    TokenKind::Percent => {
      match program.get_scope(i, TokenKind::Percent) {
        Ok((tokens, index)) => {
          if tokens.len() != 1 {
            return Err("Inventory requirements must be in the format '%var%'".to_string())
          }
          let var = match &tokens[0].kind {
            TokenKind::Text(t) => tokens[0].clone(),
            other => return Err("Incorect type in iventory requirement, must be in the format '%var%'".to_string()),
          };
          // TODO Check if correct
          let inventory = InventoryKind::Global;
          let action = InventoryAction::Check;
          let item = GameItem {
            name: var,
            action: action,
            inventory: inventory,
          };
          Ok((ParseNode::new(Expr::Require(item)), index))
        },
        Err(msg) => Err(msg),
      }
    },
    TokenKind::Pipe => {
      match program.get_scope(i, TokenKind::Pipe) {
        Ok((tokens, index)) => {
          if tokens.len() != 1 {
            return Err("Expected '|' to be appart of '|BREAK|'".to_string())
          }
          let keyword = match &tokens[0].kind {
            TokenKind::Keyword(t) => {
              if t == "BREAK" {
                t
              } else {
                return Err(format!("Expected to find the keyword 'BREAK' inside '|..|', but found '|{}|' instead", t))
              }
            },
            TokenKind::Text(t) => {
              if t.starts_with("DELAY") {
                return Ok((ParseNode::new(Expr::Delay(tokens[0].clone())), index))
              } else {
                return Err(format!("Expected to find the keyword 'BREAK' inside '|..|', but found '|{}|' instead", t))
              }
            }
            other => return Err(format!("Expected to find the keyword 'BREAK' inside '|..|', but found '|{}|' instead", tokens[0].to_string())),
          };

          Ok((ParseNode::new(Expr::Break), index))
        },
        Err(msg) => Err(msg),
      }
    },
    TokenKind::Plus => Err("Found an unexpected '+'".to_string()),
    TokenKind::Pound => {
      match program.get_scope(i, TokenKind::Pound) {
        Ok((tokens, index)) => {
          for tok in tokens.iter() {
            match &tok.kind {
              TokenKind::Text(t) => (),
              other => return Err(format!("Bolded text '#..#' only supports string objects however {} was found", token_kind_to_string(&other))),
            }
          }
          let game_text = GameText {
            text: tokens,
            itallic: false,
            bold: true,
            color: 0,
          };
          Ok((ParseNode::new(Expr::Text(game_text)), index))
        },
        Err(msg) => Err(msg),
      }
    },
    TokenKind::Semicolon => Err("Found an unexpected ';'".to_string()),
    TokenKind::Tilde => {
      match program.get_scope(i, TokenKind::Tilde) {
        Ok((tokens, index)) => {
          for tok in tokens.iter() {
            match &tok.kind {
              TokenKind::Text(t) => (),
              other => return Err(format!("Italliciesed text '~..~' only supports string objects however {} was found", token_kind_to_string(&other))),
            }
          }
          let game_text = GameText {
            text: tokens,
            itallic: true,
            bold: false,
            color: 0,
          };
          Ok((ParseNode::new(Expr::Text(game_text)), index))
        },
        Err(msg) => Err(msg),
      }
    },
    TokenKind::Newline => Err("ICE: parse_token passed a newline token".to_string()),
  }
)}

fn parse_pipe(program: &Program, pos: usize) -> Result<(ParseNode, usize), String> {
  match program.get_scope(pos, TokenKind::Pipe) {
    Ok((tokens, index)) => {
      if tokens.len() == 0 || !match_token_kind(&tokens[0].kind, &TokenKind::Keyword("BREAK".to_string())) {
        return Err(format!("Unexpected symbols found inside of '|...|', found: {}", tokens.into_iter().map(|t| -> String { t.to_string() }).collect::<String>() ))
      } else {
        return Ok((ParseNode::new(Expr::Break), index))
      }
    },
    Err(msg) => return Err(msg),
  }
}

fn parse_inventory_modification(program: &Program, pos: usize) -> Result<(ParseNode, usize), String> {
  if program.tokens[pos].kind != TokenKind::Plus && program.tokens[pos].kind != TokenKind::Minus {
    return Err(format!("ICE: Starting parse_inventory_modification on wrong token, found '{}' instead", program.tokens[pos].to_string()));
  }
  let positive = if program.tokens[pos].kind == TokenKind::Plus { true } else { false };
  let mut new_pos = pos;
  let mut count = 0;
  if positive {
    while program.tokens[new_pos].kind == TokenKind::Plus {
      count = count + 1;
      new_pos = new_pos + 1;
    }
  } else {
    while program.tokens[new_pos].kind == TokenKind::Minus {
      count = count + 1;
      new_pos = new_pos + 1;
    }
  }

  let inventory_kind = match count {
    1 => InventoryKind::Personal,
    2 => InventoryKind::Room,
    3 => InventoryKind::Global,
    other => return Err(format!("Was expecting no more than 3 {}, but {} where found", if positive { "+" } else { "-" }, other)),
  };
  let inventory_action = if positive { InventoryAction::Add } else { InventoryAction::Remove };

  let modification = GameItem {
    inventory: inventory_kind,
    action: inventory_action,
    name: program.tokens[new_pos].clone(),
  };

  Ok((ParseNode::new(Expr::Modify(modification)), new_pos+1))
}

fn parse_caret(program: &Program, pos: usize) -> Result<(ParseNode, usize), String> {
  if program.tokens[pos].kind != TokenKind::Caret {
    return Err(format!("ICE: Starting parse_carrot on wrong token, found '{}' instead", program.tokens[pos].to_string()));
  }

  match program.get_scope(pos, TokenKind::Caret) {
    Ok((tokens, index)) => {
      if tokens.len() == 0 {
        return Err(format!("Unexpected symbols found inside of '|...|', found: {}", tokens.into_iter().map(|t| -> String { t.to_string() }).collect::<String>()))
      } else {
        match program.tokens[pos+1].kind {
          TokenKind::Plus | TokenKind::Minus => {
            match parse_inventory_modification(&program, pos+1) {
              Ok((node, new_pos)) => {
                if new_pos != index-1 {
                  return Err(format!("Unexpected tokens in ^...^: {}", program.tokens[new_pos..index-1].into_iter().map(|t| -> String { t.to_string() }).collect::<String>()))
                } else {
                  return Ok((node, index))
                }
              },
              Err(msg) => return Err(msg)
            }
          },
          _ => return Err(format!("Unexpected token '{}' inside of ^...^ expression", program.tokens[pos+1].to_string()))
        }
        return Ok((ParseNode::new(Expr::Break), index))
      }
    },
    Err(msg) => return Err(msg),
  }
}

fn parse_scope(program: &Program, start_pos: usize, end_pos: usize) -> Result<Vec<ParseNode>, String> {
  if start_pos == end_pos {
    return Err(format!("Recived an empty scope"))
  }

  //print!("Scope-Start index {}: {}\nScope-End index {}: {}\n", start_pos, program.tokens[start_pos].to_string(), end_pos, program.tokens[end_pos].to_string());
  let mut nodes: Vec<ParseNode> = Vec::new();
  let mut i = start_pos+1;
  while i < (end_pos-1) {
    //print!("i: {}\ntoken: {}\n\n", i, program.tokens[i].to_string());
    match program.tokens[i].kind {
      TokenKind::Newline => i = i + 1,
      //TokenKind::OpenCurlyBrace => return Err("Unexpected '{{' encountered!".to_string()),
      //TokenKind::CloseCurlyBrace => return Err("Unexpected '}}' encountered!".to_string()),
      //TokenKind::Pipe => {
      //  i = match parse_pipe(&program, i) {
      //    Ok((node, new_index)) => {
      //      nodes.push(node);
      //      new_index
      //    },
      //    Err(msg) => return Err(msg),
      //  }
      //},
      //TokenKind::Caret => {
      //  i = match parse_caret(&program, i) {
      //    Ok((node, new_index)) => {
      //      nodes.push(node);
      //      new_index
      //    },
      //    Err(msg) => return Err(msg),
      //  }
      //}
      //_ => return Err(format!("TODO: {}", program.tokens[i].to_string())),
      _ => {
        i = match parse_token(&program, i) {
          Ok((node, new_index)) => {
            nodes.push(node);
            new_index
          },
          Err(msg) => return Err(msg),
        }
      }
    }
  }
  Ok(nodes)
}

fn parse_section(program: &Program, pos: usize, token: String) -> Result<(ParseNode, usize), String> {
  let mut new_pos = pos;
  //let name = match program.check_token(new_pos, TokenKind::Keyword("ROOM".to_string()))
  let name = match program.check_token(new_pos, TokenKind::Keyword(token.to_string()))
    .and_then(|i| program.eat_whitespace_tokens(i))
    .and_then(|i| program.get_scope(i, TokenKind::OpenSquareBracket)) {
    Ok((room_name, index)) => {
      new_pos = index;
      if room_name.len() != 1 {
        //return Err(format!("Expected 1 token for room name but found {}", room_name.len()))
        return Err(format!("Expected 1 token for {} name but found {}", token, room_name.len()))
      } else {
        room_name[0].clone()
      }
    },
    Err(msg) => return Err(msg),
  };
  let mut requirements: Vec<GameItem> = Vec::new();
  while program.tokens[new_pos].kind != TokenKind::OpenCurlyBrace {
    match &program.tokens[new_pos].kind {
      TokenKind::Newline => new_pos = new_pos + 1,
      TokenKind::Ampersand | TokenKind::Percent | TokenKind::Dollar => {
        let item = match parse_token(&program, new_pos) {
          Ok((node, i)) => {
            new_pos = i;
            match &node.value {
              Expr::Require(gameItem) => gameItem.clone(),
              _ => return Err(format!("ICE: Got {} when a Require Expression was expected", expr_to_string(&node.value))),
            }
          },
          Err(msg) => return Err(msg),
        };
        requirements.push(item);
      },
      other => return Err(format!("Unexpected token '{}' found in parameter requirements", program.tokens[new_pos].to_string())),
    }
  }
  let scope_start = match program.find_next_token(TokenKind::OpenCurlyBrace, new_pos) {
    Some(pos) => pos,
    None => return Err(format!("Expected '{} |{}|' statement to have a scope", token, name.to_string())),
  };

  let scope = match program.get_scope(new_pos, TokenKind::OpenCurlyBrace) {
    Ok((tokens, i)) => {
      //print!("Found {}: {}\n", token, name.to_string());
      //print!("TOKENS:");
      //for token in &tokens {
      //  print!("{}", token.to_string());
      //}
      new_pos = i;
      tokens
    },
    Err(msg) => return Err(msg),
  };
  if scope.len() == 0 {
    return Err(format!("Found an empty scope"));
  }
  let scope_end = match program.find_prev_token(TokenKind::CloseCurlyBrace, new_pos) {
    Some(pos) => pos,
    None => return Err(format!("'{} |{}|' is missing a '}}' token", token, name.to_string())),
  };

  let scope_nodes = match parse_scope(&program, scope_start, scope_end) {
    Ok(nodes) => nodes,
    Err(msg) => return Err(msg),
  };
  
  //print!("\n\n");
  if token == "ROOM" {
    let room = GameRoom {
      name: name,
      requirements: requirements,
      scope: scope_nodes,
    };
    return Ok((ParseNode::new(Expr::Room(room)), new_pos));
  } else {
    let action = GameAction {
      action: program.tokens[pos].clone(),
      name: name,
      requirements: requirements,
      scope: scope_nodes,
    };
    return Ok((ParseNode::new(Expr::Action(action)), new_pos));
  }
}

fn parse(program: &Program) -> Result<Vec<ParseNode>, String> {
  let mut pos = 0;
  let mut nodes: Vec<ParseNode> = Vec::new();
  match program.eat_whitespace_tokens(pos) {
    Ok(i) => pos = i,
    Err(msg) => return Err(msg),
  }
  while pos < program.tokens.len() {
    match &program.tokens[pos].kind {
      TokenKind::Keyword(t) if (t == "ROOM" || t == "HELP" || t == "MISC" || t == "EXAMINE" || t == "USE" || t == "TAKE" || t == "TALK" || t == "GO") => {
        match parse_section(&program, pos, t.to_string()) {
          Ok((n, i)) => {
            nodes.push(n);
            pos = i;
          },
          Err(msg) => return Err(msg),
        }
      },

      TokenKind::Newline => pos += 1,
      _ => {
        let (row, col) = program.get_location_with_token(&program.tokens[pos]);
        let line = program.get_line_with_token(&program.tokens[pos]);
        let token = program.tokens[pos].to_string();
        //let count = if token.chars().next().unwrap() == '"' {token.chars().count()-2} else {token.chars().count()};
        let count = token.chars().count();
        let highlight = format!("{}{}", util::repeat(col, ' '), util::repeat(count, '^'));
        print!("ERROR INFO: {}", pos);
        return Err(format!("Found incorect token on line {} while parsing {} or TODO\n{}\n{}", row+1, token, line, highlight))
      },
    }
    //pos += 1
  }

  //Err("Finished parsing".to_string())
  Ok(nodes)
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

