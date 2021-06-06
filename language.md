# Language

## BASM file format

```
<Program Name>
<Program Version>
<Strings section marker, optional>
<string definitions, optional>
<Data section marker, optional>
<data definitions, optional>
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
prts answer
prtc ' '
prt acc
```

Results in `Answer:7`

> :warning: The whole tape file is limited to 65535 bytes. This includes ops, strings, and data.

#### Comments

Any text following a `#` is ignored

```asm
#this whole line is ignored
add acc 1 #only this part is ignored
```

### Strings

The tape file can include strings that can be easily printed using `PRTS`. 

```asm
.strings
example=This is a string.
.ops
prtd example
```

The strings can't be indexed or accessed in any other way

### Data

The tape file can include byte arrays of data. Each data line must be an array of arrays and must be a max of 255 per sub array and 255 sub arrays. The input can be bytes, characters or strings (that will be converted to a byte array):
```asm
.data
squares=[[1,4,9,25,36]]
hex=[[x45,xFF]]
text=["str1", "str2", "str3"] # this is actually [[115, 116, 114, 49], [115...
letters=["abcdef"]
.ops
ld a0 squares 0 0 #squares[0][0]
prt a0
ld a0 squares 0 2 #squares[0][2]
prt a0
prtln
ld a0 text 0 0 #text[0][0]
prtc a0
ld a0 text 0 0 #text[0][1]
prtc a0
prtln
ld a0 text 0 0 #text[0][0]
prt a0
prtln
cpy acc 6
ld a1 letters 0 0 
pstr a1
ld a0 squares 0 4
cpy acc a0
inc acc
prt acc
```

Outputs
```
19
st
115
abcdef
37
```

## Assembly

* Mnemonics and registers are case insensitive
* Device is big endian

#### Params 

- `data_reg`: `acc`, `d0`, `d1`, `d2`, `d3`
- `addr_reg`: `a0`, `a1`  
- `num`: `0`-`255` or `x0`-`xFF` or ASCII in the form `'c'`
- `addr`: `@0`-`@65535` or `@x0`-`@xFFFF` 
- `lbl`: `[a-zA-Z][a-zA-Z0-9_]*`
- `data`: `[a-zA-Z][a-zA-Z0-9_]*(\[\d+\])+`

#### Constants

In the ops section constants can be defined like this
`const <name> <value>`

The name can not be the same as any label, mnemonic or register.

The value must be a valid parameter and the definition must come before any use.

### Math

`ADD data_reg data_reg|num|addr_reg`

Sets `ACC` = 1st param + 2nd param
If 2nd param is address reg then it must be pointing at the data section

`SUB data_reg data_reg|num|addr_reg`

Sets `ACC` = 1st param - 2nd param
If 2nd param is address reg then it must be pointing at the data section

`INC data_reg|addr_reg`

Increment 1st param
If 1st param is address reg then it must be pointing at the data section

`DEC data_reg|addr_reg`

Decrement 1st param
If 1st param is address reg then it must be pointing at the data section

### Data

```
CPY
data_reg data_reg
addr_reg addr_reg
data_reg data_reg addr_reg
addr_reg data_reg data_reg
addr_reg addr|label
data_reg num
data_reg data
```

Copies values from right to left, in most cases from 2nd param to 1st param. Except with `addr_reg` where it's to/from `addr_reg` and both `data_reg`. 

`MEMR addr|addr_reg`

Read byte from `addr` in memory and set in `ACC`

`MEMW addr|addr_reg`

Read from `ACC` and set byte `addr` in memory

`SWP data_reg|addr_reg data_reg|addr_reg`

Swap values in both registers, data can only be used with data and addr with addr.

`MEMC addr_reg addr_reg`

Copy `ACC` bytes from 1st param in data to 2nd param in memory

`LD addr_reg data_key data_reg|num data_reg|num`

Load 1st param with address of byte(4th param) of array(3rd param) of data(2nd param)

`LEN data_key data_reg|num`

Load `ACC` with length of array(2nd param) of data(1st param)


### Printing

`PRT data_reg|num|addr_reg`

Print value
If 1st param is address reg then it must be pointing at the data section

`PRTC data_reg|num|addr_reg`

Print value as character
If 1st param is address reg then it must be pointing at the data section

`PRTS string_name`

Print string from tape strings

`PRTLN`

Go to new line

`MEMP addr|addr_reg`

Print `ACC` characters from addr in memory

`PRTD addr_reg`

Print `ACC` characters from addr in data

### Comparison

`CMP data_reg|addr_reg addr_reg|data_reg|num`

Compare 1st param and 2nd param and set result in `ACC`
If 1st param is a data_reg and 2nd param is an addr_reg then the 2nd param will be used as an address for data

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

`FOPEN data_reg|num`

Opens input file <1st param> for reading, populates `D3`-`D0` with file size in bytes

`FILER data_reg|num addr|addr_reg`

Reads up to `ACC` bytes from <1st param> file cursor and save at `addr` in memory, populates `ACC` with number of bytes actually read

`FILEW data_reg|num addr|addr_reg`

Reads up to `ACC` bytes from memory starting `addr` in memory to <1st param> file cursor, populates `ACC` with number of bytes actually written

`FSKIP data_reg|num data_reg`

Skip up to `reg` bytes in <1st param> file, populates `ACC` with number of bytes actually skipped

`FSEEK data_reg|num`

Move <1st param> file cursor to `[D3][D2][D1][D0]` (0..4294967295)

`FCHK data_reg|num addr|addr_reg|label`

Jump to addr if a input file <1st param> is available 

### Bits

`AND reg reg|num|addr_reg`

and bits in 1st param and 2nd param and store in `ACC`

`XOR reg reg|num|addr_reg`

xor bits in 1st param and 2nd param and store in `ACC`

`OR reg reg|num|addr_reg`

or bits in 1st param and 2nd param and store in `ACC` 

`NOT reg`

Invert bits in 1st param

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

Prevents the device executing any further and terminates the program

`RAND reg`

Generate a pseudorandom number (`0`-`255`) and put in 1st param

`SEED reg`

Set the rng seed

`TIME`

Populates `D0` with seconds, `D1` with minutes, `D2` with hours