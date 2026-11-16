# XOR

Performs `binary XOR (^)` operation on two same-size numbers. Result is stored in `!A` for 8-bit values and `?LL1` for 64-bit values

**Operands:**
- `Immediate`, `Immediate`
- `LongImmediate`, `LongImmediate`
- `Register`, `Register`
- `LongRegister`, `LongRegister`
- `Immediate`, `Register`
- `Register`, `Immediate`
- `LongRegister`, `LongImmediate`
- `LongImmediate`, `LongRegister`
- `LongRegister`, `Address`

**Example:**
```armasm
xor !A 3
xor ?LL1 &12
xor ?LL1 ?LL2
xor !A !X
```
