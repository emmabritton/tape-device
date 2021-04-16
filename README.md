# Tape Device

Language for fictional tape based computer with no screen or input devices that executes asm programs optionally with input data.

#### Hardware

- Registers: ACC, D0, D1, D2, D3
- RAM: 65535 bytes
- Max program length: 21,845
- Max 

### Info

Example program
```
CPY D0 1
CPY D1 1
ADD D0 D1
PRT ACC
```

Prints `2`

### Usage

Execute program
```
tape-device program.cart <input.dat>
```

Compile program
```
tape-device compile program.tasm data.str
```

Decompile program
```
tape-device decompile program.tape
```
