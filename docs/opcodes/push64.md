# Push64

Pushes 64-bit value onto the stack

**Operands:**
- `LongImmediate`
- `LongRegister`

**Example:**
```armasm
push64 ?LL1
push64 &10
pop64  ; 10 is in the ?LL1
pop64  ; Previous state of the ?LL1 is back in the ?LL1
```
