# JZ

Jumps (moves PC) to the specified label or address only if specified register is **equal to zero**, allowing to create branching and loops

**Operands:**
- `Register/LongRegister`, `Address/IndirectAddress`

**Example:**
```armasm
mov 0 !G1  ; setup the counter
loop:
; Check if !G1 is equal to target value (let's say 10)
sub !G1 10
jz !A $loop_end

; <Do something in a loop>

; Increment the G1
add !G1 1
mov !A !G1
jmp $loop

loop_end:
```
This corresponds to this C code:
```C
for (int G1 = 0; G1 != 10; G1++) {
    // <Do something in a loop>
}
```
