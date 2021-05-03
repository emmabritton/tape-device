# Tape Device

Language for fictional computer with no screen or input devices that executes programs optionally with input data.

### Device

- Registers: 
  - Special: ACC (Accumulator)
  - 8 bit: D0, D1, D2, D3
  - 16 bit: A0, A1
  - Internal (not directly accessible):
    - FP: Frame pointer
    - SP: Stack pointer
    - Overflow: Overflow flag
    - PC: Program counter
- RAM: 65,535 bytes
- Max program length: 65,535 bytes
- Max string data: 65,535 bytes (max length per string: 255 bytes)

## Info

Example program
```
CPY D0 1
CPY D1 2
ADD D0 D1
PRT ACC
```

Prints `3`

## Usage

**Execute program**
```
tape-device program.tape [input]
```

**Assemble program**
```
tape-device assemble program.tasm
```

**Decompile program**
```
tape-device decompile program.tape
```

**Debug program** (Must be run from a terminal)
```
tape-device debug program.tape [input]
```