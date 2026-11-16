# NOT

Performs `binary NOT (!)` operation on the number. Result is stored in `!A` for 8-bit value and `?LL1` for 64-bit value

**Operands:**
- `Immediate`
- `LongImmediate`
- `Register`
- `LongRegister`

**Example:**
```armasm
not 25
not &25
not !A
not ?LL1
```
