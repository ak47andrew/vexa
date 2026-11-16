# SHL

Performs `binary left shift (<<)` operation. First operand is the value and second operand is the shit amount. Result is stored in `!A` for 8-bit values and `?LL1` for 64-bit values

**Operands:**
- `Immediate`, `Immediate`
- `Register`, `Immediate`
- `Immediate`, `Register`
- `Register`, `Register`
- `LongImmediate`, `Immediate`
- `LongImmediate`, `Register`
- `LongRegister`, `Immediate`
- `LongRegister`, `Register`
- `LongRegister`, `LongRegister`
- `LongRegister`, `LongImmediate`
- `LongImmediate`, `LongRegister`
- `LongImmediate`, `LongImmediate`

**Example:**
```armasm
shl !A 3
shl ?LL1 10
shl ?LL1 ?LL2
shl !A !X
shl ?LL1 !A
```
