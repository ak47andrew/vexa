# JNZ

Jumps (moves PC) to the specified label or address only if specified register is **NOT equal to zero**, allowing to create branching and loops

**Operands:**
- `Register/LongRegister`, `Address/IndirectAddress`

**Example:**
```armasm
; Check against 30
sub !G1 30
; If result isn't zero - then the initial value wasn't 30, meaning we need to jump to `else` block
jnz !A $else_block

; <Do something, G1 = 30>

jmp $if_finished  ; So we're not executing the else block

else_block:

; <Do something else, G1 != 30>

; No need to jump here - if_finished is right away

if_finished:
```
This corresponds to this C code:
```C
if (G1 == 30) {
    // <Do something, G1 = 30>
} else {
    // <Do something else, G1 != 30>
}
```
