## Debugger

Debug a program

### Usage

Must be run with a tape file and a debug file

`./tape_device debug program.tape program.debug <input>`

### Keys

|Key|Use|Note|
|---|---|----|
|\<space>|Step||
|<ctrl+c>|Quit||
|\<escape>|Leave text entry mode, stop auto-run or quit|
|h|Help|Prints help|
|i|Info|Prints debugger state info|
|b|Set breakpoint|Set a breakpoint|
|u|Clear breakpoint|Clear a breakpoint|
|8|Toggle 8bit dec/hex|Toggles showing 8 bit values between decimal and hexadecimal|
|6|Toggle 16bit dec/hex|Toggles showing 16 bit values between decimal and hexadecimal|
|l|Toggle parsed line|Toggles between parsed and original source line|
|c|Toggle chars|Toggle between showing chars for registers|
|y|History|Print execution history, this prints to the output area and will not be cleared|
|s|Input string|Enter up to 255 characters, submitted when return is pressed|
|t|Input char|Enter any letter, symbol, number, \<space>, \<tab>, \<return>, \<delete>, \<backspace>, \<escape>. To send escape press shift+escape as escape will stop the input mode|

