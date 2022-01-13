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
answer="Answer: "
.ops
cpy d0 5
cpy d1 3
add d0 d1
sub acc 1
prts answer
prt acc
```

Results in `Answer: 7`

> :warning: Each section is limited to 65535 bytes

#### Comments

Any text following a `#` is ignored

```asm
#this whole line is ignored
add acc 1 #only this part is ignored
```

Comments are not supported on string or data definitions

```asm
.data
#this is fine
nums=[[10,20]] #this is not
```

### Strings

The tape file can include strings that can be easily printed using `PRTS`. 

```asm
.strings
example=This is a string.
.ops
prts example
```

The strings are trimmed, to include whitespace place the string double quotes:
```asm
spaced="  this line has 2 two spaces either side  "
```

This will print without the quotes, if you want quotes in the string use two quotes:
```asm
a_quote=""Sphinx of black quartz, judge my vow""
```

This will print with one quote on either side.

Comments are be treated as part of the string

The strings can't be indexed or accessed in any other way

### Data

The tape file can include byte arrays of data. Each data line must be an array of arrays and must be a max of 255 bytes per sub array and 254 sub arrays. The input can be bytes, characters or strings (that will be converted to a byte array):
```asm
.data
squares=[[1,4,9,25,36]]
hex=[[x45,xFF]]
text=["str1", "str2", "str3"] # this is actually [[115, 116, 114, 49], [115...
letters=["abcdef"]
bin=[[b00000001,b00000010]]
.ops
```

See `LD` for more information

## Assembly

* Mnemonics, keywords and registers (but not section dividers) are case insensitive

#### Params 

- `data_reg`: `acc`, `d0`, `d1`, `d2`, `d3`
- `addr_reg`: `a0`, `a1`  
- `num`: `0`-`255` or `x0`-`xFF` or ASCII char `'c'` or `b00000000`
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

`DEC data_reg|addr_reg`

Decrement 1st param

### Data

```
CPY
data_reg data_reg
addr_reg addr_reg
data_reg data_reg addr_reg
addr_reg data_reg data_reg
addr_reg addr|label
data_reg num
data_reg areg
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

Memory is packaged as such:

`<array count> <array1 length> <array2 length> <array1 bytes> <array2 bytes>`

This allows length to be retrieved like this:

```asm
.data
list=[[10,11],[97,98,99]]
.ops
LD A0 lists 0 0
```

Packaged: `02 02 03 0A 0B 61 62 63`

| offset 1 | offset 2 | desc | value |
|----------|----------|-------|-----|
| 0 | 0 | array count | 2 |
| 0 | 1 | length of array 1 | 2 |
| 0 | 2 | length of array 2 | 3 |
| 0 | 3+ | invalid | err |
| 1 | 0 | byte 1 of array 1 | 10 |
| 1 | 1 | byte 2 of array 1 | 11 |
| 1 | 2+ | invalid | err |
| 2 | 0 | byte 1 of array 2 | 97 |
| 2 | 1 | byte 2 of array 2 | 98 |
| 2 | 2 | byte 3 of array 2 | 99 |
| 2 | 3+ | invalid | err |
| 3+ | * | invalid | err |

### Printing

`PRT data_reg|num|addr_reg`

Print value

If param is address reg then it must be pointing at the data section

`PRTC data_reg|num|addr_reg`

Print value as character

If param is address reg then it must be pointing at the data section

`PRTS string_name`

Print string from tape strings

`PRTLN`

Go to new line

`MEMP addr|addr_reg`

Print `ACC` characters from addr in memory

`PRTD addr_reg`

Print `ACC` characters from addr in data

General usage:
```asm
.data
text=["an example"]
.ops
ld a0 text 0 1   #get addr of length of 'an example'
cpy acc a0       #load length into acc
ld a0 text 1 0   #get addr of 'an example'
prtd a0
```

### Comparison

`CMP data_reg|addr_reg addr_reg|data_reg|num`

Compare 1st param and 2nd param and set result in `ACC`
If 1st param is a data_reg and 2nd param is an addr_reg then the 2nd param will be used as an address for data
ACC will contain 0 if equal, 1 if LHS < RHS, 2 if LHS > RHS
Use JE, JNE, JL, JG to act on result

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

Jump to label if overflow flag is set

`NOVER lbl|addr_reg`

Jump to label if overflow flag is not set

### File

`FOPEN data_reg|num`

Opens input file <1st param> for reading, populates `D3`-`D0` with file size in bytes

`FILER data_reg|num addr|addr_reg`

Reads up to `ACC` bytes from <1st param> file cursor and save at `addr` in memory, populates `ACC` with number of bytes actually read

`FILEW data_reg|num addr|addr_reg|data_reg|num`

Reads up to `ACC` bytes from memory starting `addr` in memory to <1st param> file cursor or writes literal value from reg or num, populates `ACC` with number of bytes actually written

`FSKIP data_reg|num data_reg`

Skip up to `reg` bytes in <1st param> file, populates `ACC` with number of bytes actually skipped

`FSEEK num`

Move <1st param> file cursor to `[D3][D2][D1][D0]` (0..4294967295)

`FSEEK data_reg`

Move <1st param> file cursor to the latest 4 value in the stack (0..4294967295)

Internally this calls `POP ACC` 4 times, first popped/last pushed is low byte.

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

Invert bits in 1st param and store in `ACC` 

### Stack

`CALL addr_reg|label|addr`

Jumps to address provided 

`RET`

Jumps to instruction after last executed `CALL`

`PUSH reg|num`

Push value from register or number on to stack.

`POP reg`

Pop value from stack and populates register.

`ARG reg|addr_reg num|reg`

Get 1 or 2 bytes (depending on if 1st param is reg or addr reg) from 2nd param bytes before the frame pointer

To get the first argument call `ARG <reg> 1`, if the first param is 1 byte then the second param is `ARG <reg> 2` otherwise call `ARG <reg> 3` and so on

This instruction does not alter data on the stack or move the SP or FP.

*See examples/stack_example.basm for more info*

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

`DEBUG`

Prints system dump, similar to system crash
