class Shell
{
    constructor(canvas_width, canvas_height, fonts, text_size)
    {
        this.shell_line = "> ";
        this.width = canvas_width;
        this.height = canvas_height;
        if (fonts.regular !== undefined) {
          this.font = fonts.regular;
        } else {
          this.font = fonts;
        }
        if (fonts.bold !== undefined) {
          this.bold = fonts.bold;
        } else {
          this.bold = this.font;
        }
        if (fonts.italic !== undefined) {
          this.italic = fonts.italic;
        } else {
          this.italic = this.font;
        }
        this.text_size = text_size;
        this.command_ready = false;
        this.continue = false;
    }

    // Grabs character when a key is pressed
    keyTyped()
    {
        //      Space                     Digits                           Upper-Case                       Lower-Case
        if((keyCode == 32) || (keyCode > 47 && keyCode < 58) || (keyCode > 64 && keyCode < 91) || (keyCode > 96 && keyCode < 123))
        {
            this.shell_line += key;
        } else
        {
            switch(keyCode)
            {
                case BACKSPACE: case DELETE:  // Backspace
                if(this.shell_line.length > 2)
                    this.shell_line = this.shell_line.slice(0, this.shell_line.length - 1);
                console.log("BACK");
                break;

                case RETURN: // Newline
                console.log("Enter pressed");
                this.command_ready = true;
                break;

//                case DELETE:
//                this.continue = true; 
//                console.log("DELETE");
//                break;

                default:
                // Pass
                console.log("Unhandled key pressed " + keyCode.toString());
            }
        }
    }

    commandReady()
    {
        return this.command_ready;
    }

    getCommand()
    {
        this.command_ready = false;
        var ret_val = this.shell_line.slice(2, this.shell_line.length).toLowerCase();
        while(ret_val[0] == ' ')
            ret_val = ret_val.slice(1, ret_val.length);
        ret_val = ret_val.split(' ');
        this.shell_line = "> ";
        console.log("RETURN VALUE");
        console.log(ret_val);
        return ret_val;
    }

    // Draws the line
    draw()
    {
        push();
        text(this.shell_line, (-this.width/2) + 100, (this.height / 2) - 50);//10, this.height - 50);
        pop();
    }


}
