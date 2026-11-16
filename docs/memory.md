# Memory

There's quite a few places where data is stored and read, we're gonna visit them one at a time. Starting with...

## Main memory

So basically, it's your main memory chip, accessible with addresses (prefixed with `$` in assembly). And it's split into multiple logical parts. And since they're only logical, it's easy to overflow some addresses and start writing to Stack while thinking you're still in RAM

### RAM

It starts at address `$0` and spreads for 10 * 1024 bytes (10 KB), ending at `$10240`

This is basically your workspace: you can use it to store variables, manipulate bytes in any way, etc.

For example to store player's score you can decide on address (let's say 15) and then update when needed:

```armasm
update_score:
    mov $15 ?LL1
    add ?LL1 &1
    mov ?LL1 $15
    ret
```

### Stack

This is the part where data goes on `push` and `pop` commands as well as calling the function with `call` and returning with `ret`. For example

```armasm
; Pushing values onto the stack
push 2
push 3
push 4
push 5

; Popping into !A
pop  ; moves 5 to !A
log !A

pop  ; moves 4 to !A
log !A

pop
log !A

pop
log !A
```

It's especially useful if you're working with functions (this example is a bit complex, but I believe in ya <3)

```armasm
jmp $start

factorial:   ; Calculates factorial of a number stored in ?LL1: 1 * 2 * ... * ?LL1
    jz ?LL1 $base_case  ; If ?LL1 is 0, return 1 to break recursion, 0! = 1
    push64 ?LL1  ; Store current value so we can store it later
    sub ?LL1 &1 ; ?LL1 - 1, stored in ?LL1 reg (sub always saves into ?LL1)
    call $factorial  ; Call with lowered value (?LL1 - 1)
    mov ?LL1 ?GP1  ; pop sets ?LL1 so we need to save the return value
    pop64  ; Getting initial ?LL1 back
    mul ?LL1 ?GP1  ; ?LL1 * factorial(?LL1 - 1)
    ret  ; Mul sets to ?LL1 which is all we need
base_case:
    mov &1 ?LL1
    ret  ; We jumped here and not called, so last address on the stack is still from $factorial function

start:
    ; so, to call factorial, we need to set ?LL1 as an argument and call

    mov &10 ?LL1
    call $factorial
    ; Result is also at ?LL1 so we can... idk, log it
    log ?LL1

    exit
```

Stack starts were `SP` is initialized which is at address $10241 and spreads for 2 kilobytes

> Yes, I know that it's usually shares the space with RAM, but... I just decided to do it this way... Okay, okay, this is just because it was easier to implement this way. And anyway, I created it, I win 😝

## Program memory

Program lives in a different "memory chip" entirely. The data in `.bin` file copied there, `?PC` is set to 0 and execution starts from there. While executing RAM or modifying program at runtime was possible in VEA, due to here it's being in the different address space entirely, it's not possible in VEXA

## Registers

Registers are like variables you can use during your runtime. There's two types of them depending on their size: 8-bit (Register) and 64-bit (LongRegister) ones 


Here's a cheatsheet for what commands edit what registers:
| instruction |                            register writes                            |
| :---------: | :-------------------------------------------------------------------: |
|     add     | `!A` (8 bit), `?LL1` (64) - result, `!X` (8), `?LL2` (64) - overflow  |
|     and     |                     8-bit - `!A`, 64-bit - `?LL1`                     |
|    call     |                             `?PC`, `?SP`                              |
|    copy     |                                   -                                   |
|     div     | `!A` (8 bit), `?LL1` (64) - result, `!X` (8), `?LL2` (64) - remainder |
|    exit     |                                   -                                   |
|     ext     |                Whatever register is at second operand                 |
|     hlt     |                                   -                                   |
|     jmp     |                                 `?PC`                                 |
|     jnz     |                                 `?PC`                                 |
|     jz      |                                 `?PC`                                 |
|    line     |                                   -                                   |
|     log     |                                   -                                   |
|     mov     |                Whatever register is at second operand                 |
|     mul     | `!A` (8 bit), `?LL1` (64) - result, `!X` (8), `?LL2` (64) - overflow  |
|    noop     |                                   -                                   |
|     not     |                     8-bit - `!A`, 64-bit - `?LL1`                     |
|     or      |                     8-bit - `!A`, 64-bit - `?LL1`                     |
|     pop     |                              `!A`, `?SP`                              |
|    pop64    |                             `?LL1`, `?SP`                             |
|    push     |                                 `?SP`                                 |
|   push64    |                                 `?SP`                                 |
|     ret     |                             `?PC`, `?SP`                              |
|     shl     |                     8-bit - `!A`, 64-bit - `?LL1`                     |
|     shr     |                     8-bit - `!A`, 64-bit - `?LL1`                     |
|     sub     | `!A` (8 bit), `?LL1` (64) - result, `!X` (8), `?LL2` (64) - underflow |
|    trunc    |                Whatever register is at second operand                 |
|     xor     |                     8-bit - `!A`, 64-bit - `?LL1`                     |

> Note: VEXA doesn't have flags! at all! !Z, !X might look like them, but it's a register like any other. 

> Note: If not written in the command, register state persists between instructions and call/ret/jump boundaries. If you write `add` and then `div`, div won't reset `!Z` and it will stay at whatever state `add` left it.

### Register

Registers (8-bit) are prefixed with `!` and split into two different categories:
- Program space: `!A`, `!X`, `!Y`, `!Z` - this ones can be touched by different instructions. For example `div` saves the result to the `!A` register and the remainder to the `!X`
- User space: `!G1` to `!G5` - these one are completely there for your comfort, only modified by explicit `mov`. Drop a thing there and it's gonna wait for ya when you'll need it

### LongRegister

Long Registers (64-bit) are prefixed with `?` and follow the same schema:
- Program space: `?LL1`, `?LL2`
- User space: `?GP1` to `?GP3`
- Special registers: there's also two LongRegisters that're used elsewhere
    - `?PC` (Program counter): it points to the address that's gonna be executed at the next step
    - `?SP` (Stack pointer): handles where to read and write stack to
