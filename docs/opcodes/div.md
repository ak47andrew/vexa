# DIV

Divides two numbers of the same size and stores result in one register and remainder in another. For 8-bit values it's `!A` and `!X`, for 64-bit it's `?LL1` and `?LL2`. Dividing by zero leads to program crash

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
div !A 3
div ?LL1 &12
div ?LL1 ?LL2
div !A !X
```
