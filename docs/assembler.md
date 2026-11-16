# Assembly language

Assembly language is something you're gonna be working with. A lot. So pay attention, kids, this is really important

Assembly language here is called VEXA which stands for "Extended VEA", which was my previous project.

Overall the language is pretty much your usual assembly language. Instructions are split with newlines, first word is the instruction (or opcode) - basically what you need to do. Other are operands (arguments) that specifies sources, destinations, sizes, dev's mother phone numbers and so on.

For example `mov` will tell it to move the data, first operand `!A` will tell it to pull value from register `A` and second operand `$0` will tell it to move it to RAM

## Comments

This only supports one-line comments and they start with semicolon (`;`)

No multiline for ya, sorry!

```armasm
; This is the comment!
hlt  ; I can also be up here too!
; And you can't break the comment ; exit - nope, not gonna work
```

Unofficial docstrings convention for functions (which I personally use) is this:
```armasm
;; ---
;; remove_bullet(n) -> void
;; Takes n in !G1 - index of the bullet
;; Removes bullet at nth index
;; ---
remove_bullet:
    ext !G1 ?LL1
    mul ?LL1 &4
    add ?LL1 $20
    mov 0 [?LL1]
    ret
```

## Labels

Actually feasible way of doing addresses! Jumping to `$30` is... strange and unreliable. So what can you do instead?

Introducing: Labels! Just drop it at any line of your code with `name:` and you can jump to it instantly - compiler is gonna do the heavy lifting

```armasm
infinite_loop:

jmp $infinite_loop  ; See? Much better than `jmp $30`!
```
