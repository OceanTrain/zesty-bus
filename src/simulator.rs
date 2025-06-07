pub mod game;
pub mod inventory;

use std::collections::HashMap;
use std::{thread, time};
use wasm_bindgen::prelude::*;

use crate::parser::lex;
use crate::parser::parse;
use crate::parser::read_program;
use crate::parser::read_program_from_string;
use crate::parser::ast::ParseNode;
use crate::parser::lex::Expr;
use crate::parser::lex::expr_to_string;
use crate::parser::lex::GameAction;
use crate::parser::lex::GameRoom;

use game::Game;
use inventory::Inventory;

pub fn run() {
  let root_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let www_path = root_path.join("www");
  let narrative_path = www_path.join("narrative.txt");
  let mut program = read_program(&narrative_path);
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
#[allow(dead_code)]
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
  let current_room = String::from("init");
  let (mut room, mut actions) = match find_room(&rooms, &current_room, &inventory) {
    Ok((room, actions)) => (room, actions),
    Err(msg) => return Err(msg),
  };

  let mut line = String::new();
  loop {
    println!("Now in Room: {}", room.name.to_string());
    let mut skip_actions = false;
    for i in 0..room.scope.len() {
      match &room.scope[i].value {
        Expr::Break => { 
          print!("\n");
          match std::io::stdin().read_line(&mut line) {
            Ok(_) => (),
            Err(msg) => return Err(msg.to_string()),
          }
          line.clear();
        },
        Expr::Delay(_token) => { thread::sleep(time::Duration::from_secs(5)); },  // TODO: actually read delay.
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
        Expr::Audio(_game_audio) => { continue; },
        Expr::Action(game_action) => { return Err(format!("Discovered Action '{} |{}|' inside of Room '{}'", game_action.action.to_string(), game_action.name.to_string(), current_room)); },
        Expr::Require(game_item) => { return Err(format!("Discovered 'Require({})', inside of Room '{}'", game_item.name.to_string(), current_room)); },
        Expr::Modify(game_item) => { inventory = inventory.modify(&game_item, &current_room); },
      }
    }

    if skip_actions {
      continue;
    }


    let mut leave_actions = false;
    while !leave_actions {

      print!("\n");
      let index: usize;
      loop {
        let mut command: String = "MISC".to_string();
        let argument: String;
        loop {
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
            match std::io::stdin().read_line(&mut line) {
              Ok(_) => (),
              Err(msg) => return Err(msg.to_string()),
            }
            line.clear();
          },
          Expr::Delay(_token) => { thread::sleep(time::Duration::from_secs(5)); },  // TODO: actually read delay.
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
          Expr::Audio(_game_audio) => { continue; },
          Expr::Action(game_action) => { return Err(format!("Discovered Action '{} |{}|' inside of Action '{} |{}|'", game_action.action.to_string(), game_action.name.to_string(), action.action.to_string(), action.name.to_string())); },
          Expr::Require(game_item) => { return Err(format!("Discovered 'Require({})', inside of Action '{} |{}|'", game_item.name.to_string(), action.action.to_string(), action.name.to_string())); },
          Expr::Modify(game_item) => { inventory.modify(&game_item, &current_room); },
        }
      }
    }
  }
}

fn setup_rooms(nodes: &Vec<ParseNode>) -> Result<HashMap<String, (Vec<GameRoom>, Vec<GameAction>)>, String> {
  let mut rooms = HashMap::<String, (Vec<GameRoom>, Vec<GameAction>)>::new();
  let mut grouped_rooms = Vec::<GameRoom>::new();
  let mut grouped_actions = Vec::<GameAction>::new();
  let mut current_room_name: String = "".to_string();
  for i in 0..nodes.len() {
    match &nodes[i].value {
      Expr::Room(game_room) => {
        if game_room.name.to_string() != current_room_name {
          if !grouped_rooms.is_empty() {
            if rooms.contains_key(&current_room_name) {
              return Err(format!("The room name '{}' was found in multiple differnt sections", current_room_name));
            }
          }

          rooms.insert(current_room_name.clone(), (grouped_rooms.clone(), grouped_actions.clone()));
          grouped_rooms.clear();
          grouped_actions.clear();
          current_room_name = game_room.name.to_string();
        }
        grouped_rooms.push(game_room.clone());
      },
      Expr::Action(game_action) => {
        grouped_actions.push(game_action.clone());
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


