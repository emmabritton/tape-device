# Tape Device

Language (vm, assembler, and decompiler) for fictional computer that executes programs optionally with input data.

### Device

- Registers:
  - 8 bit: D0, D1, D2, D3, ACC (Accumulator)
  - 16 bit: A0, A1
  - Internal (not directly accessible):
    - FP: Frame pointer
    - SP: Stack pointer
    - Overflow: Overflow flag
    - PC: Program counter
- RAM: 65,535 bytes
- Max ops length: 65,535 bytes 
- Max strings length: 65,535 bytes (max 255 bytes per string)
- Max data length: 65,535 bytes (max 254 sub arrays per data definition, max 255 bytes per sub array)

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
If you're having build issues add `--save-intermediate` to save the assemblers interpretation
For the debugger add `--save-debug` to save debug data

**Decompile program**
```
tape-device decompile program.tape
```

## TODO

- [ ] gui debugger
    - [ ] register dump
    - [ ] breakpoints
    - [ ] step-by-step
    - [ ] data viewer
    - [ ] memory viewer
    - [ ] code insertion/editing?