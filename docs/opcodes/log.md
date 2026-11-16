# LOG

Outputs all information about specific object into terminal output

**Operands:**
- `<Any (single operand)>`

**Example:**
```armasm
log $0  ; [DEBUG] Address: $0; 8-bit value: 0; 64-bit value: 0
log 100  ; [DEBUG] 100
log &500  ; [DEBUG] 500
log ^1000000000  ; [DEBUG] 1000000000
log !G1  ; [DEBUG] REG(G1) = 0
log ?PC  ; [DEBUG] LONG_REG(PC) = 36
log [?PC]  ; [DEBUG] LONG_REG(PC) = Address: $39; 8-bit value: 0; 64-bit value: 0
exit
```
