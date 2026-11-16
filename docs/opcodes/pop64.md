# Pop64

Pop 64-bit value off of the stack and stores it in the `?LL1` register

**Operands:**
- `none`

**Example:**
```armasm
push64 ?LL1
push64 &10
pop64  ; 10 is in the ?LL1
pop64  ; Previous state of the ?LL1 is back in the ?LL1
```
