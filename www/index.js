import * as wasm from "reflections";

function init() {
  window.global.var.narrative = window.global.var.narrative.join(" ");
  window.global.var.game = wasm.compile(window.global.var.narrative);
  let start_text = window.global.var.game.start();
  window.global.var.displayString = new printClass(start_text, -900, -500, 1600);

  window.global.func.draw = draw;
  draw();
}

function draw() {
  // Main game loop
  background(0);
  let time = Math.floor(millis());
  if (window.global.var.displayString !== undefined) { 
    window.global.var.displayString.oprint();
  }
  window.global.var.shell.draw();
  if(window.global.var.shell.continue)
  {
    window.global.var.shell.continue = false;
  }
  
}

function keyTyped() {
    //parser.interupt();
    window.global.var.shell.keyTyped();
    if(window.global.var.shell.commandReady()) {
        executeCommand();
    }


}

function audioCallback(sound)
{
     sound.play()
}

function executeCommand()
{
  var command = window.global.var.shell.getCommand();
  if(command[0] == "") {
    window.global.var.displayString = new printClass(window.global.var.game.print_current_room(), -900, -500, 1600);
    return;
  }
  //shell.shell_line = "> ";
  var command_list = ["examine", "use", "talk", "go", "take", "help", "inventory"];
  var short_command_list = ["e", "u", "l", "g", "t", "h", "y", "n"];
  
  var index = command_list.indexOf(command[0]);
  if(index === undefined)
      index = short_command_list.indexOf(commandi[0]);
  if(index === undefined)
  {
      // TODO: error
  }

  var return_val;
  console.log("Index is: " + index);
  switch(index)
  {
    case 0:  // Examine
    return_val = window.global.var.game.query("EXAMINE", command[1]);
    break;

    case 1:  // Use
    return_val = window.global.var.game.query("USE", command[1]);
    break;

    case 2:  // Talk
    return_val = window.global.var.game.query("TALK", command[2]);
    break;

    case 3:  // Go
    return_val = window.global.var.game.query("GO", command[1]);
    break;

    case 4:  // Take
    return_val = window.global.var.game.query("TAKE", command[1]);
    break;

    case 5:  // Help
    return_val = window.global.var.game.query("HELP", "help");
    break;

    case 6:  // Inventory
    //parser.query("INVENTORY", "inventory");
    return_val = window.global.var.game.print_inventory();
    break;

    default:
    return_val = window.global.var.game.query("MISC", command[0]);
  }

  console.log("Query returned: " + return_val);
  //window.global.var.displayString = new printClass(return_val, -450, -250, 900);
  window.global.var.displayString = new printClass(return_val, -900, -500, 1600);
}

function soundEffect(sound)
{
  sound.play();
}

function backgroundMusic(sound)
{
  console.log("trying to play bgm");
  if (bgm != undefined && bgm.isPlaying()) {
    console.log("ambience is playing, trying to replace it");
    bgm.stop();
  }
  bgm = sound;
  console.log("BGM is now: ", bgm);
              sound.setLoop(true);
              sound.play();
}

window.global.func.draw = init;
window.global.func.keyTyped = keyTyped;
