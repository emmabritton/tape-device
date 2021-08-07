# Tape Device

Language (vm, assembler, debugger, and decompiler) for fictional computer that executes programs optionally with input data.

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
tape_device program.tape [input0] [input1]...
```

**Assemble program**
```
tape_device assemble program.basm
```
If you're having build issues add `--save-intermediate` to save the assemblers interpretation.
For a debugger add `--save-debug` to save debug data

**Decompile program**
```
tape_device decompile program.tape
```

**Debug program**
```
tape_device debug program.tape program.debug [input]
```
[Debug docs](https://github.com/raybritton/tape-device/blob/master/debug_device.md)
