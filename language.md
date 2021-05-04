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

The value must be a valid parameter

### Math

`ADD reg data_reg|num`

Sets `ACC` = param1 + param2

`SUB reg data_reg|num`

Sets `ACC` = param1 - param2

### Data

`CPY data_reg data_reg|num`

Sets param1 = param2

`MEMR addr|addr_reg`

Read byte from `addr` in memory and set in `ACC`

`MEMW addr|addr_reg`

Read from `ACC` and set byte `addr` in memory

### Printing

`PRT data_reg|num`

Print value

`PRT addr|addr_reg`

Print value as character

`PRTD addr|addr_reg`

Print string from tape data

`PRTLN`

Go to new line

### Comparison

`CMP data_reg data_reg|num`

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

### Addresses

`SWPAR`

Swap contents of `A0` and `A1`

`CMPAR`

Compares contents of `A0` and `A1` and stores result in `ACC`
Values are relative to `A0`, so if `A0` > `A1` then `ACC` will contain `GREATER`

`CPYA0 addr|lbl or reg reg`

Copy from parameters (address, or two data registers) into `A0`

`CPYA1 addr|lbl or reg reg`

Copy from parameters (address, or two data registers) into `A1`

`LDA0 reg reg`

Copy value from `A0` into registers

`LDA1 reg reg`

Copy value from `A1` into registers

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

### Misc

`NOP`

Do nothing

`HALT`

Prevents the device executing any further and in normal operation terminates the program