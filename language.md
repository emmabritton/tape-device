# Language

## Assembly

#### Params 

- `reg`: `acc`, `d0`, `d1`, `d2`, `d3`
- `num`: `0`-`255` or `x0`-`xFF`
- `addr`: `x0`-`xFFFF` 
- `lbl`: `[a-zA-Z0-9_]+`

### Math

`ADD reg reg|num`

Sets `ACC` = param1 + param2

`SUB reg reg|num`

Sets `ACC` = param1 - param2

### Data

`CPY reg reg|num`

Sets param1 = param2

`MEMR addr`

Read byte from `addr` in memory and set in `ACC`

`MEMW addr`

Read from `ACC` and set byte `addr` in memory

### Printing

`PRT reg|num`

Print value

`PRT addr`

Print value as character

`PRTD addr`

Print string from tape data

`PRTLN`

Go to new line

### Comparison

`CMP reg reg|num`

Compare param1 and param2 and set result in `ACC`

### Jump

`JMP lbl`

Jump to label

`JE lbl`

Jump to label if last two compared values were equal, based on value in `ACC`

`JNE lbl`

Jump to label if last two compared values were not equal, based on value in `ACC`

`JL lbl`

Jump to label if last compared lhs was less than rhs, based on value in `ACC`

`JG lbl`

Jump to label if last compared lhs was greater than rhs, based on value in `ACC`

`OVER lbl`

Jump to label if overflow flag set

`NOVER lbl`

Jump to label if overflow flag not set

### File

`FOPEN`

Opens specified input file for reading, populates D3-D0 with file size in bytes

`FREAD addr`

Reads up to `ACC` bytes from file cursor and save at `addr` in memory, populates `ACC` with number of bytes actually read

`FWRITE addr`

Reads up to `ACC` bytes from memory starting `addr` in memory to file cursor, populates `ACC` with number of bytes actually written

`FSKIP reg`

Skip up to `reg` bytes in file, populates `ACC` with number of bytes actually skipped

`FSEEK`

Move file cursor to `[D3][D2][D1][D0]` (0..4294967295)

### Misc

`NOP`

Do nothing

##