# JMP

Jumps (moves PC) to the specified label or address, allowing to create infinite loops or skipping some parts of the code

**Operands:**
- `Address`
- `IndirectAddress`

**Example:**
```armasm
loop:
; <Do something in an endless loop>

jmp $loop
```
