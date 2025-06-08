use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use crate::parser::ast::ParseNode;
use crate::parser::lex::Expr;
use crate::parser::lex::GameAction;
use crate::parser::lex::GameRoom;
use crate::simulator::inventory::Inventory;

#[wasm_bindgen]
#[derive(Clone)]
pub struct GameState {
  inventory: Inventory,
  current_room_name: String,
  current_room_index: usize,
}

impl GameState {
  pub fn new(inventory: Inventory, room_name: String, room_index: usize) -> GameState {
    GameState {
      inventory: inventory,
      current_room_name: room_name,
      current_room_index: room_index,
    }
  }

  pub fn init() -> GameState {
    GameState::new(Inventory::new(), String::from("init"), 0)
  }

  pub fn get_inventory(&mut self) -> &mut Inventory {
    &mut self.inventory
  }

  pub fn get_room_name(&self) -> &String {
    &self.current_room_name
  }

  pub fn get_room_index(&self) -> usize {
    self.current_room_index
  }

  pub fn set_inventory(&mut self, inventory: Inventory) {
    self.inventory = inventory;
  }

  pub fn set_room_name(&mut self, name: String) {
    self.current_room_name = name; 
  }

  pub fn set_room_index(&mut self, index: usize) {
    self.current_room_index = index;
  }

  pub fn eq(&self, other: &GameState) -> bool {
    self.current_room_index == other.current_room_index
      && self.current_room_name == other.current_room_name
      && self.inventory.eq(&other.inventory)
  }
}

#[wasm_bindgen]
pub struct GameResult {
  text: String,
  state: GameState,
}

impl GameResult {
  pub fn new(text: String, state: GameState) -> GameResult {
    GameResult {
      text: text,
      state: state,
    }
  }
}

#[wasm_bindgen]
impl GameResult {
  pub fn to_string(&self) -> String {
    self.text.clone()
  }

  pub fn to_state(&self) -> GameState {
    self.state.clone()
  }
}

#[wasm_bindgen]
pub struct Game {
  rooms: HashMap<String, (Vec<GameRoom>, Vec<GameAction>)>,
}

impl Game {
  pub fn new(rooms_map: &HashMap<String, (Vec<GameRoom>, Vec<GameAction>)>) -> Game {
    let mut new_map = HashMap::new();
    for (key, val) in rooms_map.iter() {
      let (r, a) = val;
      new_map.insert(key.to_string(), (r.clone(), a.clone()));
    }

    Game {
      rooms: new_map,
    }
  }

  pub fn print_room(&self, room_name: &String, state: &GameState) -> String {
    let room = match self.find_room(&room_name, &state) {
      Ok((r, _a)) => r,
      Err(msg) => panic!("{}", msg),
    };

    let mut output = "{\n  \"room_output\": [\n".to_string();
    for i in 0..room.scope.len() {
      match &room.scope[i].value {
        Expr::Break => { 
          output.push_str("    \"|BREAK|\",\n");
        },
        Expr::Delay(token) => { output.push_str(&format!("    \"|{}|\",\n", token.to_string())); },  // TODO: actually read delay.
        Expr::Room(game_room) => { panic!("Discovered Room '{}' inside of Room '{}'", game_room.name.to_string(), state.get_room_name()); },
        Expr::Goto(token) => {
          output.push_str(&format!("    \"[[{}]]\",\n", token.to_string()));
        },
        Expr::Text(game_text) => {
          output.push_str(&format!("    \"{}\",\n", format!("{} ", (&game_text.text).into_iter().map(|t| -> String { t.to_string() }).collect::<String>())));
        },
        Expr::Audio(game_audio) => { output.push_str(&format!("    \"<{}>\",\n", game_audio.path.to_string())); },
        Expr::Action(game_action) => { panic!("Discovered Action '{} |{}|' inside of Room '{}'", game_action.action.to_string(), game_action.name.to_string(), state.get_room_name()); },
        Expr::Require(game_item) => { panic!("Discovered 'Require({})', inside of Room '{}'", game_item.name.to_string(), state.get_room_name()); },
        Expr::Modify(game_item) => { 
          //self.inventory.modify(&game_item, &self.current_room_name); 
          output.push_str(&format!("    \"^{}^\",\n", game_item.name.to_string()));
        },
      }
    }
    output.push_str("  ],\n}");

    output
  }

  pub fn find_room(&self, room_name: &String, state: &GameState) -> Result<(GameRoom, Vec<GameAction>), String> {
    match self.rooms.get(room_name) {
      Some((game_rooms, game_actions)) => {
        for i in 0..game_rooms.len() {
          if state.inventory.check_items(&game_rooms[i].requirements, &room_name) {
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

  pub fn find_action_index(&self, action_type: &String, action_name: &String, state: &GameState) -> Result<usize, String> {
    let actions = match self.rooms.get(state.get_room_name()) {
      Some((_r, a)) => a,
      None => return Err(format!("ICE: Could not find the room '{}'", state.get_room_name())),
    };

    for i in 0..actions.len() {
      if actions[i].action.to_string().to_uppercase() == action_type.to_string().to_uppercase() && actions[i].name.to_string().to_lowercase() == action_name.to_string().to_lowercase() {
        if state.inventory.check_items(&actions[i].requirements, &state.get_room_name()) {
          return Ok(i);
        }
      }
    }
    
    return Err(format!("Invalid command '{} {}', try again", action_type, action_name));
  }

  pub fn print_scope(&self, scope: &Vec<ParseNode>, state: &GameState) -> GameResult {
    let mut output = String::new();
    let mut new_state = state.clone();
    let mut inventory = new_state.get_inventory().clone();
    for i in 0..scope.len() {
      match &scope[i].value {
        Expr::Break => { 
          output.push_str("|BREAK|\n");
        },
        Expr::Delay(token) => { output.push_str(&format!("|{}|\n", token.to_string())); },  // TODO: actually read delay.
        Expr::Room(game_room) => { panic!("Discovered Room '{}' inside of Room '{}'", game_room.name.to_string(), new_state.get_room_name()); },
        Expr::Goto(token) => {
          let new_room_name = token.to_string();
          match self.find_room(&new_room_name, &state) {
            Ok((r, _a)) => {
              new_state.set_room_name(r.name.to_string());
            },
            Err(msg) => return GameResult::new(format!("Error: {}", msg), new_state),
          };
        },
        Expr::Text(game_text) => {
          if i+1 < scope.len() {
            match &scope[i+1].value {
              Expr::Text(_t) => output.push_str(&format!("{} ", (&game_text.text).into_iter().map(|t| -> String { t.to_string() }).collect::<String>())),
              _ => output.push_str(&format!("{}\n", format!("{} ", (&game_text.text).into_iter().map(|t| -> String { t.to_string() }).collect::<String>()))),
            }
          } else {
            output.push_str(&format!("{}\n", format!("{} ", (&game_text.text).into_iter().map(|t| -> String { t.to_string() }).collect::<String>())));
          }
        },
        Expr::Audio(game_audio) => { output.push_str(&format!("<{}>\n", game_audio.path.to_string())); },
        Expr::Action(game_action) => { panic!("Discovered Action '{} |{}|' inside of Room '{}'", game_action.action.to_string(), game_action.name.to_string(), new_state.get_room_name()); },
        Expr::Require(game_item) => { panic!("Discovered 'Require({})', inside of Room '{}'", game_item.name.to_string(), new_state.get_room_name()); },
        Expr::Modify(game_item) => { 
          inventory = inventory.modify(&game_item, &new_state.get_room_name()); 
        },
      }
    }
    new_state.set_inventory(inventory);

    GameResult::new(output, new_state)
  }

  pub fn get_current_room(&self, state: &GameState) -> GameRoom {
    self.rooms.get(state.get_room_name()).unwrap().0[state.get_room_index()].clone()
  }

  fn _run_room(&self, state: &GameState) -> (GameState, bool, bool) {
    let mut room_change = false;
    let mut inventory_change = false;
    let mut new_state = state.clone();
    let mut inventory = new_state.get_inventory().clone();
    let room = match self.find_room(&state.get_room_name(), &state) {
      Ok((r, _a)) => r,
      Err(msg) => panic!("{}", msg),
    };
    for i in 0..room.scope.len() {
      match &room.scope[i].value {
        Expr::Room(game_room) => { panic!("Discovered Room '{}' inside of Room '{}'", game_room.name.to_string(), new_state.get_room_name()); },
        Expr::Goto(token) => {
          let new_room_name = token.to_string();
          match self.find_room(&new_room_name, &state) {
            Ok((r, _a)) => {
              new_state.set_room_name(r.name.to_string());
              room_change = true;
            },
            Err(msg) => panic!("Error: {}", msg),
          };
        },
        Expr::Action(game_action) => { panic!("Discovered Action '{} |{}|' inside of Room '{}'", game_action.action.to_string(), game_action.name.to_string(), new_state.get_room_name()); },
        Expr::Require(game_item) => { panic!("Discovered 'Require({})', inside of Room '{}'", game_item.name.to_string(), new_state.get_room_name()); },
        Expr::Modify(game_item) => { 
          inventory = inventory.modify(&game_item, &new_state.get_room_name()); 
          inventory_change = true;
        },
        _ => (),
      }
    }
    new_state.set_inventory(inventory);
    (new_state, room_change, inventory_change)
  }
}

#[wasm_bindgen]
impl Game {
  pub fn start(&self) -> GameResult {
    let state = GameState::init();
    let rooms = match self.rooms.get(&state.current_room_name) {
       Some((r, _a)) => r,
       None => return GameResult::new(format!("Error: ROOM |{}| not found", state.get_room_name()), state),
     };

    for i in 0..rooms.len() {
      if state.inventory.check_items(&rooms[i].requirements, &state.get_room_name()) {
        return self.print_scope(&rooms[i].scope, &state);
      }
    }

    let text = format!("Error: Could not find any ROOM |{}| for which satisfied the current inventory requirements\n{}", state.current_room_name, state.inventory.to_string(&state.get_room_name()));
    GameResult::new(text, state)
  }

  pub fn list_all_rooms(&self, state: &GameState) -> GameResult {
    let text = self.rooms.keys().fold(String::new(), |a, b| a + b + "\n");
    GameResult::new(text, state.clone())
  }

  pub fn print_current_room(&self, state: &GameState) -> GameResult {
    self.print_scope(&self.get_current_room(&state).scope, &state)
  }

  pub fn query(&self, action: String, command: String, state: &GameState) -> GameResult {
    let index = match self.find_action_index(&action, &command, &state) {
      Ok(i) => i,
      Err(msg) => return GameResult::new(format!("Could not find action: {} |{}| under ROOM |{}|. ({})", action, command, state.get_room_name(), msg), state.clone()),
    };

    let scope = &self.rooms.get(state.get_room_name()).unwrap().1[index].scope;

    self.print_scope(&scope, &state)
  }

  pub fn print_inventory(&self, state: &GameState) -> GameResult {
    let text = state.inventory.to_string(&state.get_room_name()).to_string();
    GameResult::new(text, state.clone())
  }
}


