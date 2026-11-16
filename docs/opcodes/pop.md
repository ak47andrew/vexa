# Pop

Pops 8-bit value off of the stack and stores it in the `!A` register

**Operands:**
- `none`

**Example:**
```armasm
push !A
push 10
pop  ; 10 is in the !A
pop  ; Previous state of the !A is back in the !A
```
