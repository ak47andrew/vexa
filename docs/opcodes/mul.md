# MUL

Multiply two numbers of the same size and stores result in a register. For 8-bit values it's `!A`, for 64-bit - `?LL1`. If overflow occurred, `!Z` is set to `1`, otherwise it's set to `0`

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
mul !A 3
mul ?LL1 &12
mul ?LL1 ?LL2
mul !A !X
```
