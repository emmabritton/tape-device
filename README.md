# Tape Device

Language (vm, compiler, decompiler) for fictional computer that executes programs optionally with input data.

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
- Max program length: 65,535 bytes (includes data and strings)

## Info

Example program
```
CPY D0 1
CPY D1 2
ADD D0 D1
PRT ACC
```

Prints `3`

See [docs](https://github.com/raybritton/tape-device/blob/master/language.md) for more info

## Usage

**Execute program**
```
tape-device program.tape [input0] [input1]...
```

**Assemble program**
```
tape-device assemble program.basm
```

**Decompile program**
```
tape-device decompile program.tape
```
