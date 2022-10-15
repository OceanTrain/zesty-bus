#[wasm_bindgen]
#[derive(Clone)]
pub struct Inventory {
  personal: HashSet<String>,
  room: HashMap<String, HashSet<String>>,
  global: HashSet<String>,
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

    //format!("{}\n{}\n{}\n", personal, room, global)
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

  pub fn add_item(&self, item: &GameItem, room_name: &String) -> Inventory {
    match item.inventory {
      InventoryKind::Personal => { 
        let mut personal = self.personal.clone();
        let global = self.global.clone();
        let room = self.room.clone();
        personal.insert(item.name.to_string()); 
        Inventory {
          personal: personal,
          global: global,
          room: room,
        }
      },
      InventoryKind::Global => { 
        let personal = self.personal.clone(); 
        let mut global = self.global.clone(); 
        let room = self.room.clone();
        global.insert(item.name.to_string()); 
        Inventory {
          personal: personal,
          global: global,
          room: room,
        }
      },
      InventoryKind::Room => {
        let personal = self.personal.clone(); 
        let global = self.global.clone(); 
        let mut room = self.room.clone();
        match room.get_mut(room_name) {
          Some(room_inventory) => { room_inventory.insert(item.name.to_string()); },
          None => {
            let mut new_inventory = HashSet::new();
            new_inventory.insert(item.name.to_string());
            room.insert(room_name.to_string(), new_inventory);
          },
        }
        Inventory {
          personal: personal,
          global: global,
          room: room,
        }
      },
    }
  }

  pub fn remove_item(&self, item: &GameItem, room_name: &String) -> Inventory {
    match item.inventory {
      InventoryKind::Personal => { 
        let mut personal = self.personal.clone(); 
        let global = self.global.clone(); 
        let room = self.room.clone();
        personal.remove(&item.name.to_string()); 
        Inventory {
          personal: personal,
          global: global,
          room: room,
        }
      },
      InventoryKind::Global => { 
        let personal = self.personal.clone(); 
        let mut global = self.global.clone(); 
        let room = self.room.clone();
        global.remove(&item.name.to_string()); 
        Inventory {
          personal: personal,
          global: global,
          room: room,
        }
      },
      InventoryKind::Room => {
        let personal = self.personal.clone(); 
        let global = self.global.clone(); 
        let mut room = self.room.clone();
        match room.get_mut(room_name) {
          Some(room_inventory) => { room_inventory.remove(&item.name.to_string()); },
          None => (),
        }
        Inventory {
          personal: personal,
          global: global,
          room: room,
        }
      }
    }
  }

  pub fn modify(&self, item: &GameItem, room: &String) -> Inventory {
    match item.action {
      InventoryAction::Add => self.add_item(&item, &room),
      InventoryAction::Remove => self.remove_item(&item, &room),
      InventoryAction::Check => panic!("ICE: Attempting to modify a check item '{}'", item.name.to_string()),
    }
  }

  fn rooms_eq(&self, other: &Inventory) -> bool {
    self.room.len() == other.room.len() 
      && self.room.keys().all(|k| other.room.contains_key(k))
      && self.room.keys().all(
           |k| self.room.get(k).expect("ICE: Unreachable") == other.room.get(k).expect("ICE: Unreachable")
         )
  }

  pub fn eq(&self, other: &Inventory) -> bool {
    self.personal == other.personal
      && self.global == other.global
      && self.rooms_eq(other) 
  }
}
