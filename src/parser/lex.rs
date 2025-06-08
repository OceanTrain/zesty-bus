use crate::parser::tokenize;
use crate::parser::ParseNode;

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
  pub item: tokenize::Token,
}


#[derive(Debug, Clone)]
pub struct GameText {
  pub text: Vec<tokenize::Token>,
  pub itallic: bool,
  pub bold: bool,
  pub color: u32,
}

#[derive(Debug, Clone)]
pub struct GameAudio {
  pub path: tokenize::Token,
  pub sound_effect: bool,
}

#[derive(Debug, Clone)]
pub struct GameItem {
  pub name: tokenize::Token,
  pub action: InventoryAction,
  pub inventory: InventoryKind,
}

#[derive(Debug, Clone)]
pub struct GameAction {
  pub action: tokenize::Token,
  pub name: tokenize::Token,
  pub requirements: Vec<GameItem>,
  pub scope: Vec<ParseNode>,
}

#[derive(Debug, Clone)]
pub struct GameRoom {
  pub name: tokenize::Token,
  pub requirements: Vec<GameItem>,
  pub scope: Vec<ParseNode>,
}


// The more complex grammar expressions
#[derive(Debug, Clone)]
pub enum Expr {
  Break,
  Delay(tokenize::Token),
  Room(GameRoom),
  Goto(tokenize::Token),
  Text(GameText),
  Audio(GameAudio),
  Action(GameAction),
  Require(GameItem),
  Modify(GameItem),
}

pub fn expr_to_string(expr: &Expr) -> String {
  match expr {
    Expr::Break => format!("|BREAK|"),
    Expr::Delay(token) => format!("{}", tokenize::token_kind_to_string(&token.kind)),
    Expr::Room(game_room) => format!("Room |{}|", game_room.name.to_string()),
    Expr::Goto(token) => format!("[[{}]]", tokenize::token_kind_to_string(&token.kind)),
    Expr::Text(game_text) => format!("{}", (&game_text.text).into_iter().map(|t| -> String { t.to_string() }).collect::<String>()),
    Expr::Audio(game_audio) => format!("<{}>", game_audio.path.to_string()),
    Expr::Action(game_action) => format!("{} |{}|", game_action.action.to_string(), game_action.name.to_string()),
    Expr::Require(game_item) => format!("REQUIRE({})", game_item.name.to_string()),
    Expr::Modify(game_item) => format!("MODIFY({})", game_item.name.to_string()),
  }
}

