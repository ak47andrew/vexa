# Ext
Move data from lower-size container to the higher-size one. This can also be used to transfer one byte from main memory to LongRegister instead of the whole 8 when using `mov`

**Operands:**
- `Address`, `LongRegister`
- `IndirectAddress`, `LongRegister`
- `Immediate`, `LongRegister`
- `Register`, `LongRegister`

**Example:**
```armasm
ext $0 ?LL1  ; Just one byte
mov $0 ?LL1  ; whole 0-7 range
ext !A ?LL2
```