# Language

## Assembly

#### Params 

- `data_reg`: `acc`, `d0`, `d1`, `d2`, `d3`
- `addr_reg`: `a0`, `a1`  
- `num`: `0`-`255` or `x0`-`xFF`
- `addr`: `x0`-`xFFFF` 
- `lbl`: `[a-zA-Z0-9_]+`

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

`ADDRH addr_reg data_reg`

Load value from `data_reg` into high byte of `addr_reg`

`ADDRL addr_reg data_reg`

Load value from `data_reg` into low byte of `addr_reg`

### File

`FOPEN`

Opens specified input file for reading, populates D3-D0 with file size in bytes

`FREAD addr|addr_reg`

Reads up to `ACC` bytes from file cursor and save at `addr` in memory, populates `ACC` with number of bytes actually read

`FWRITE addr|addr_reg`

Reads up to `ACC` bytes from memory starting `addr` in memory to file cursor, populates `ACC` with number of bytes actually written

`FSKIP data_reg`

Skip up to `reg` bytes in file, populates `ACC` with number of bytes actually skipped

`FSEEK`

Move file cursor to `[D3][D2][D1][D0]` (0..4294967295)

### Misc

`NOP`

Do nothing

##