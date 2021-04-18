# Tape Device

Language for fictional computer with no screen or input devices that executes programs optionally with input data.

### Device

- Registers: ACC, D0, D1, D2, D3, A0, A1
- RAM: 65,535 bytes
- Max program length: 21,845 instructions
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
tape-device program.tape <input>
```

**Compile program**
```
tape-device compile program.tasm data.dat
```

**Decompile program**
```
tape-device decompile program.tape
```

**Debug program** (Must be run from command line)
```
tape-device debug program.tape <input>
```