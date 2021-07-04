## Known issues

### Empty Labels

#### Issue

If a program has two labels, e.g.
```asm
label1:
label2:
    inc acc
```

The first label `label1` is an empty label as it has no instruction to point to. Currently the assembler can not detect this and will generate an invalid program.

#### Fix
Comment out or remove one of the labels