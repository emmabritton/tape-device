# Language

## TASM file format

```
<Program Name>
<Program Version>
<Strings section marker, optional>
<string definitions, optional>
<Ops section marker>
<ops>
```

For example
```asm
Example Program
1
.strings
answer=Answer:
.ops
cpy d0 5
cpy d1 3
add d0 d1
sub acc 1
prtd answer
prt acc
```

Results in `Answer: 7`

#### Comments

Any text following a `#` is ignored

```asm
#this whole line is ignored
add acc 1 #only this part is ignored
```

## Assembly

* Mnemonics and registers are case insensitive
* Device is big endian

#### Params 

- `data_reg`: `acc`, `d0`, `d1`, `d2`, `d3`
- `addr_reg`: `a0`, `a1`  
- `num`: `0`-`255` or `x0`-`xFF`
- `addr`: `@0`-`@65535` or `@x0`-`@xFFFF` 
- `lbl`: `[a-zA-Z][a-zA-Z0-9_]*`

#### Constants

In the ops section constants can be defined like this
`const name value`

The name can not be the same as any label, mnemonic or register.

The value must be a valid parameter and the definition must come before any use.

### Math

`ADD reg data_reg|num`

Sets `ACC` = param1 + param2

`SUB reg data_reg|num`

Sets `ACC` = param1 - param2

### Data

```
CPY
data_reg data_reg
addr_reg addr_reg
data_reg data_reg addr_reg
addr_reg data_reg data_reg
addr_reg addr|label
data_reg num
```

Copies values from right to left, in most cases from 2nd param to 1st param. Except with `addr_reg` where it's to/from `addr_reg` and both `data_reg`. 

`MEMR addr|addr_reg`

Read byte from `addr` in memory and set in `ACC`

`MEMW addr|addr_reg`

Read from `ACC` and set byte `addr` in memory

`SWP data_reg|addr_reg data_reg|addr_reg`

Swap values in both registers, data can only be used with data and addr with addr.

### Printing

`PRT data_reg|num`

Print value

`PRT addr|addr_reg`

Print value as character

`PRTD addr|addr_reg`

Print string from tape data

`PRTLN`

Go to new line

`PSTR addr|addr_reg`

Print `ACC` characters from addr in memory

### Comparison

`CMP data_reg|addr_reg addr_reg|data_reg|num`

Compare param1 and param2 and set result in `ACC`

### Jump

`JMP lbl|addr_reg`

Jump to label

`JE lbl|addr_reg`

Jump to label if last two compared values were equal, based on value in `ACC`

`JNE lbl|addr_reg`

Jump to label if last two compared values were not equal, based on value in `ACC`

`JL lbl|addr_reg`

Jump to label if last compared lhs was less than rhs, based on value in `ACC`

`JG lbl|addr_reg`

Jump to label if last compared lhs was greater than rhs, based on value in `ACC`

`OVER lbl|addr_reg`

Jump to label if overflow flag set

`NOVER lbl|addr_reg`

Jump to label if overflow flag not set

### File

`FOPEN`

Opens specified input file for reading, populates `D3`-`D0` with file size in bytes

`FILER addr|addr_reg`

Reads up to `ACC` bytes from file cursor and save at `addr` in memory, populates `ACC` with number of bytes actually read

`FILEW addr|addr_reg`

Reads up to `ACC` bytes from memory starting `addr` in memory to file cursor, populates `ACC` with number of bytes actually written

`FSKIP data_reg`

Skip up to `reg` bytes in file, populates `ACC` with number of bytes actually skipped

`FSEEK`

Move file cursor to `[D3][D2][D1][D0]` (0..4294967295)

`FCHK addr|addr_reg`

Jump to addr if a input file is available 

### Stack

`CALL addr_reg|label|addr`

Jumps to address provided and pushes return address (pc + 1) to stack. 

`RET`

Pops last two bytes from stack and jumps to them.

`PUSH reg|num`

Push value from register or number on to stack.

`POP reg`

Pop value from stack and populates register.

`ARG reg|addr_reg num|reg`

Get 1 or 2 bytes (depending on if 1st param is reg or addr reg) from 2nd param bytes before the frame pointer

To get first call `ARG <reg> 1`, if the first param 1 byte then the second param is `ARG <reg> 2` otherwise call `ARG <reg> 3` and so on

This instruction does not alter data on the stack or move the SP or FP.

*See stack example for more info*

### Input

`IPOLL addr|addr_reg`

Jump to addr if at least one character is available to be read from keyboard

`RCHR reg`

Read one character from keyboard and set in reg, blocking

`RSTR addr|addr_reg`

Read characters from keyboard and store starting at addr in memory, reads until return is pressed or 255 characters are entered.
Stores length of string in `ACC` 

### Misc

`NOP`

Do nothing

`HALT`

Prevents the device executing any further and in normal operation terminates the program