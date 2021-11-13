class printClass{
   constructor(string,x,y,bwidth){
       //String input related
       //stringPointer and length are used to create array of letter objects and run through them
       //Counter is currently unused
       //Colour stores a color string to be used for a letter
       this.fullString = string;
       this.stringPointer = 0;
       this.counter = 0;
       this.color = '#ffffff';
       this.length = 0;

       //Cases for deciding if a letter is regular, bold, italics or coloured
       this.case = 0;
       this.bold = false;
       this.italics = false;
       this.colorTest = false;

       //Output area
       //startX and Y are where the stirng should start printing
       //currentX and currentY are where the string currently is printing
       //boxWidth is how far accross they can print
       //size is text size
       //slack is how much over the width chars are allowed to print for printing strings
       this.size = width/70;
       this.startX = x;
       this.startY = y;
       this.currentX = x;
       this.currentY = y;
       this.boxWidth = bwidth;
       this.slack = 200;

       //lettes is an array of letter objects to be printed
       this.letters = [];

       //Run through the entire length of the recieved string
       //Add each string character to its own letter object and append to letters
       for(var i = 0;i < this.fullString.length; i++){

            //Bold case check
            //call boldSet to check case
            //Increase charcter we are pointing to avoid adding # to printing list
            if(this.fullString[this.stringPointer] === '#'){
                this.boldSet();
                this.stringPointer++;
            }
            
            //Same as above except for italics case
            else if(this.fullString[this.stringPointer] === '~'){
                this.italSet();
                this.stringPointer++;
            }

            else if(this.fullString[this.stringPointer] === '@'){
                this.stringPointer++;
                this.colorSet();
            }

            //Colour case goes here but not needed yet
            
            //Append letter to letters array
            //Increase stringPointer
            //Increase length
            //Length is used so that we know the actual array length since we dont add some characters ie #, ~
            //print(this.case);
            //print(this.fullString[this.stringPointer])
            this.letters[i] = new letter(this.fullString[this.stringPointer],this.color,this.case);
            this.stringPointer++;
            this.length++;

            //
            if(this.stringPointer == this.fullString.length){
                break;
            }
       }
       //Reset string pointer to use again later if necessary
       this.stringPointer = 0;
    }

    //class print function loops, loops once it hits edges
    //goes through letters array and calls each objects print
    //Currently considers each functions bold and italics case
   oprint(){
       //Resets print location for call so we consistently print in the right place
       this.currentX = this.startX;
       this.currentY = this.startY;

       //Old code, used for printing chars one at a time on screen
       //May work?, not well if it does
       /*
        if(this.stringPointer < this.fullString.length && this.counter++ > 2){
            this.counter = 0;

            //Break test
            //Doesn't wait for player input
            //if(this.fullString[this.stringPointer] == '|'){
                //this.output += '\n \n';
                //this.stringPointer += 7;
            //}
            /*
            if(this.fullString[this.stringPointer] == '@'){
                color = '#';
                for(i = 3; i < 9; i++){
                    color += this.fullString[this.stringPointer + i];
                }
                this.colorSet(color);
            }
            
           
           var j;
           print(this.stringPointer + 's');
            for(j = 0; j < this.stringPointer; j++){
                print(j + 'j');
                this.letters[j].lprint();
            }
            this.stringPointer++;
        }
        */

        //Loops through all objects in letters and calls its print function
        var j;
        for(j = 0; j < this.length; j++)
        {

            //If letters[j] is a space object and past the margin print space and jump to next line
            // or if letter[j] is a newline then jump to the next line
            if((this.letters[j].getLetter() === " " && this.currentX >= this.startX + this.boxWidth - this.slack) || this.letters[j].getLetter() === "\n")
            {
                this.letters[j].lprint(this.currentX,this.currentY);
                this.currentX = this.startX;
                this.currentY += 35;
            }

            //Else just print character as is at current location
            //Add size of a char to currentX
            else if(j < this.length)
            {
                this.letters[j].lprint(this.currentX,this.currentY);
                this.currentX += this.size;
            }

        }
        
    }
    colorSet(){
        if(this.colorTest == true){
            this.color = '#ffffff';
            this.colorTest = false;
        }
        else{
            this.colorTest = true;
            this.color = '#';
            for(var i = 0; i < 6; i++){
                this.color += this.fullString[this.stringPointer];
                this.stringPointer++;
            }
        }
    }
    boldSet(){
        if(this.bold == true){
            this.case = 0;
            this.bold = false;
        }
        else{
            this.case = 1;
            this.bold = true;
        }
    }
    italSet(){
        if(this.italics == true){
            this.case = 0;
            this.italics = false;
        }
        else{
            this.case = 2;
            this.italics = true;
        }
    }
}

class letter{
    constructor(string,color,typing){
        this.letter = string;
        this.color = color;
        this.typing = typing;
    }

    getLetter(){
        return this.letter;
    }

    lprint(x,y){
        if(this.typing == 0){
            textFont(window.global.inconsolata);
        }
        else if(this.typing == 1){
            textFont(fontBold);
        }
        else{
            textFont(fontItal);
        }
        //print(this.color);
        //let c = color(color);
        fill(this.color);

        text(this.letter, x, y, 800);
    }
}
