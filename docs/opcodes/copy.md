# COPY

Your primary way to copy huge chunks of data between different addresses in memory. First operand tells how many bytes to copy, second is the source address, third - destination address

**Operands:**
- `Immediate/LongImmediate/LongerImmediate/Register/LongRegister`, `Address/IndirectAddress`, `Address/IndirectAddress`

**Example:**

```
; Initial chunk
mov ^0x0102030405060708 $0
; Copying it over and over a few times so this pattern is repeated for 1024 bytes
copy ^8 $0 $8
copy ^16 $0 $16
copy ^32 $0 $32
copy ^64 $0 $64
copy ^128 $0 $128
copy ^256 $0 $256
copy ^512 $0 $512
copy ^1024 $0 $1024
exit
```
