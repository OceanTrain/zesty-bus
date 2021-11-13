import * as wasm from "reflections";

console.log("Loaded");
var narrative = undefined;
var gNarrativeStarted = false;
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
var fontItal;
var game;
window.global = {};
window.global.display_string = undefined;
window.gPreload = function()
{
  inconsolata = loadFont('./Courier_Prime/Courier-Prime.ttf');
  window.global.inconsolata = inconsolata;
  fontBold = loadFont('./Courier_Prime/Courier-Prime-Bold.ttf');
  fontItal = loadFont('./Courier_Prime/Courier-Prime-Italic.ttf')
  //sound = loadSound("sounds/SFX/lanternUse.mp3");
  window.global.preloadFinished = true;
}

window.gSetup = function()
{
  if (window.global.preloadFinished !== true) {
    callAfter(50, window.gSetup);
    return;
  }
  //let width = 1000;
  //let height = 600;
  //let text_size = width / 40;
  let width = 2000;
  let height = 1200;
  let text_size = width / 50;
  let text_font = inconsolata;

  createCanvas(width, height, WEBGL);
  textFont(inconsolata);
  textSize(text_size);
  textAlign(LEFT, CENTER);

  console.log("Loading narrative file");
  loadStrings("./narrative.txt", startNarrative, narrativeLoadingIssue);


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
  window.global.setupFinished = true;
}

function startNarrative(strings) {
  narrative = strings.join(" ");
  game = wasm.compile(narrative);
  let start_text = game.start();
  window.global.display_string = new printClass(start_text, -900, -500, 1600);
  console.log("Narrative file loaded");
  gNarrativeStarted = true;
}

function narrativeLoadingIssue(err) {
  console.error("Issue loading narrative: " + err);
}

window.gDraw = function() {
  if (window.global.preloadFinished !== true || window.global.setupFinished !== true) {
    background(0);
    return;
  }

  if (!gNarrativeStarted) {
    background(0);
    return;
  }

  // Main game loop
  background(0);
  let time = Math.floor(millis());
  if (window.global.display_string !== undefined) { 
    window.global.display_string.oprint();
  }
  shell.draw();
  if(shell.continue)
  {
    shell.continue = false;
  }
  
}

window.gKeyTyped = function() {
    //parser.interupt();
    shell.keyTyped();
    if(shell.commandReady())
        executeCommand();
    //game = wasm.compile();
    //console.log(game.print_current_room())


}

function audioCallback(sound)
{
     sound.play()
}

function executeCommand()
{
  var command = shell.getCommand();
  if(command[0] == "") {
    window.global.display_string = new printClass(game.print_current_room(), -900, -500, 1600);
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
    return_val = game.query("EXAMINE", command[1]);
    break;

    case 1:  // Use
    return_val = game.query("USE", command[1]);
    break;

    case 2:  // Talk
    return_val = game.query("TALK", command[2]);
    break;

    case 3:  // Go
    return_val = game.query("GO", command[1]);
    break;

    case 4:  // Take
    return_val = game.query("TAKE", command[1]);
    break;

    case 5:  // Help
    return_val = game.query("HELP", "help");
    break;

    case 6:  // Inventory
    //parser.query("INVENTORY", "inventory");
    return_val = game.print_inventory();
    break;

    default:
    return_val = game.query("MISC", command[0]);
  }

  console.log("Query returned: " + return_val);
  //window.global.display_string = new printClass(return_val, -450, -250, 900);
  window.global.display_string = new printClass(return_val, -900, -500, 1600);
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
 
