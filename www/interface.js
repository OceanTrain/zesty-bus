//import * as wasm from "reflections";
//import {compile} from "reflections";
//import * from "index.js";
import * as wasm from "reflections";

var narrative;
//var player_inventory;
//var global_inventory;
var inconsolata;
var fontBold;
var printString = "";
var stringPointer = 0;
var txt_brk = false;
var parser;
var shell = undefined;
//var sound;
var bgm;
window.preload = function()
{
    inconsolata = loadFont('./Courier_Prime/Courier-Prime.ttf');
    fontBold = loadFont('./Courier_Prime/Courier-Prime-Bold.ttf');
    fontItal = loadFont('./Courier_Prime/Courier-Prime-Italic.ttf')

    console.log("Loading narrative file");
    narrative = loadStrings("./narrative.txt");
    console.log("Narrative file loaded");
    //sound = loadSound("sounds/SFX/lanternUse.mp3");
}

window.setup = function()
{
    let width = 1000;
    let height = 600;
    let text_size = width / 40;
    let text_font = inconsolata;

    createCanvas(width, height, WEBGL);
    textFont(inconsolata);
    textSize(text_size);
    textAlign(LEFT, CENTER);

    narrative = narrative.join(" ");
    //player_inventory = new Inventory();
    //global_inventory = new Inventory();

    //console.log("attempt parser");
    //parser = new Parser(narrative, player_inventory, global_inventory, soundEffect, backgroundMusic);
    //console.log("made parser, attempt chopping");
    //parser.chopIntoTags();
    //console.log("successfully chopped");
    //console.log("hello?");
    //console.log(parser.query);
    //parser.menu()
    //parser.query("ROOM", "init");
    //console.log("successfully queueried");
    shell = new Shell(width, height, text_font, text_size);
    /*
    parser.tokenize();
    parser.printTokens();

    // Set the starting room
    parser.rooms.current_room = parser.rooms.getRoom("init", player_inventory, global_inventory);
    console.log("Running Room");
    parser.rooms.current_room.run(player_inventory, global_inventory);
    console.log("Done running room");
    //parser.rooms.current_room.next(player_inventory, global_inventory);
    //parser.rooms.current_room.next(player_inventory, global_inventory);
    //loadSound("sounds/SFX/lanternUse.mp3", audioCallback);
    
    */
}

window.draw = function() {
    // Main game loop
    background(0);
    let time = Math.floor(millis());
    if(parser !== undefined)
        {
    parser.run();
        }
    else
    {
      console.log("GOODBYE");
    }
    shell.draw();
    if(shell.continue)
    {
       //parser.rooms.current_room.next();
        shell.continue = false;
    }
}

function keyTyped() {
    //parser.interupt();
    shell.keyTyped();
    if(shell.commandReady())
        executeCommand();
    let game = wasm.compile();
    console.log(game.print_current_room())


}

function audioCallback(sound)
{
     sound.play()
}

function executeCommand()
{
    var command = shell.getCommand();
    if(command[0] == "")
        return;
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

    //switch(index)
    //{
    //    case 0:  // Examine
    //    parser.query("EXAMINE", command[1]);
    //    break;

    //    case 1:  // Use
    //    parser.query("USE", command[1]);
    //    break;

    //    case 2:  // Talk
    //    parser.query("TALK", command[2]);
    //    break;

    //    case 3:  // Go
    //    parser.query("GO", command[1]);
    //    break;

    //    case 4:  // Take
    //    parser.query("TAKE", command[1]);
    //    break;

    //    case 5:  // Help
    //    parser.query("HELP", "help");
    //    break;

    //    case 6:  // Inventory
    //    //parser.query("INVENTORY", "inventory");
    //    parser.printInventory();
    //    break;

    //    default:
    //    parser.query("MISC", command[0]);
    //}


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
 
