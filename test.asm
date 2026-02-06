; ============================================================
; Atlas ISA - Comprehensive Test Program
; Tests: immediates, ALU, branches, labels, stack, memory,
;        peek/poke, .imm constants, and conditional branches
;
; NOTE: r0 is hardwired to zero and cannot be used as a
;       destination register. Use r1+ for all mutable values.
; ============================================================

; ----- Constants -----
UART_PORT: .imm 0x01
LED_PORT:  .imm 0x02
COUNT:     .imm 5

.export main
.export done

; ============================================================
; Entry point
; ============================================================
main:
    ; --- I-type: load immediates ---
    ldi r1, 0x00          ; accumulator = 0
    ldi r2, 0x01          ; increment value
    ldi r3, COUNT         ; loop counter (uses .imm constant)
    ldi r4, 0xFF          ; max byte value

    ; --- A-type: basic ALU ---
    add  r1, r2           ; r1 = 0 + 1 = 1
    add  r1, r2           ; r1 = 1 + 1 = 2
    sub  r1, r2           ; r1 = 2 - 1 = 1
    mov  r5, r1           ; r5 = r1 (copy)

    ; --- A-type: logic ---
    ldi  r6, 0xAA
    ldi  r7, 0x55
    and  r6, r7           ; 0xAA & 0x55 = 0x00
    ldi  r6, 0xAA
    or   r6, r7           ; 0xAA | 0x55 = 0xFF
    ldi  r6, 0xF0
    xor  r6, r7           ; 0xF0 ^ 0x55 = 0xA5
    not  r6, r6           ; ~0xA5 = 0x5A
    neg  r6, r6           ; negate

    ; --- A-type: shifts ---
    ldi  r8, 0x01
    shl  r8, r2           ; 0x01 << 1 = 0x02
    shl  r8, r2           ; 0x02 << 1 = 0x04
    shr  r8, r2           ; 0x04 >> 1 = 0x02
    rol  r8, r2           ; rotate left
    ror  r8, r2           ; rotate right

    ; --- I-type: immediate arithmetic ---
    ldi  r9, 0x10
    addi r9, 0x05         ; r9 = 0x10 + 0x05 = 0x15
    subi r9, 0x03         ; r9 = 0x15 - 0x03 = 0x12
    andi r9, 0x0F         ; r9 = 0x12 & 0x0F = 0x02
    ori  r9, 0xA0         ; r9 = 0x02 | 0xA0 = 0xA2

    ; --- S-type: stack operations ---
    push r5               ; save r5
    push r9               ; save r9
    pop  r10              ; r10 = old r9
    pop  r11              ; r11 = old r5 (should be == r5)

    ; --- A-type: compare + conditional branch ---
    cmp  r10, r9          ; compare popped value with r9
    beq  equal            ; if equal, jump (they should be equal)
    br   fail             ; otherwise something went wrong

equal:
    cmp  r11, r5          ; compare second popped value with r5
    bne  fail             ; if not equal, fail

    ; --- Unconditional branch (relative offset) ---
    br   +2               ; skip the next instruction (relative)
    br   fail             ; this should be skipped

    ; --- Test tst + bmi/bpl ---
    ldi  r1, 0x80         ; negative in signed interpretation
    tst  r1, r1           ; test (sets flags based on r1 AND r1)
    bmi  is_negative      ; branch if minus flag set
    br   fail

is_negative:
    ldi  r1, 0x01         ; positive value
    tst  r1, r1
    bpl  is_positive      ; branch if plus flag set
    br   fail

is_positive:

    ; --- M-type: memory load/store ---
    ldi  r1, 0x42         ; value to store
    ldi  r2, 0x80         ; base address
    st   r1, [r2, 0]      ; store 0x42 at address 0x80
    ld   r3, [r2, 0]      ; load back from address 0x80
    cmp  r1, r3           ; verify round-trip
    bne  fail

    ldi  r4, 0x99
    st   r4, [r2, 1]      ; store at 0x81
    ld   r5, [r2, 1]      ; load from 0x81
    cmp  r4, r5
    bne  fail             ; verify round-trip

    ; --- P-type: peek/poke ---
    ldi  r1, 0x37
    poke r1, UART_PORT    ; write 0x37 to UART port (uses .imm label)
    poke r1, LED_PORT     ; write 0x37 to LED port
    peek r2, UART_PORT    ; read back from UART port

    ; --- Using r0 as a zero source (reads are fine) ---
    add  r1, r0           ; r1 += 0 (r0 is always zero)
    cmp  r0, r0           ; compare zero with zero (sets flags)
    st   r0, [r2, 2]      ; store zero to memory
    push r0               ; push zero onto stack
    pop  r1               ; pop it back (should be 0)
    poke r0, LED_PORT     ; write zero to LED port

    ; --- Counted loop using labels ---
    ldi  r1, 0x00         ; sum = 0
    ldi  r2, 0x01         ; i = 1
    ldi  r3, COUNT        ; limit

sum_loop:
    add  r1, r2           ; sum += i
    addi r2, 0x01         ; i++
    cmp  r2, r3           ; compare i with limit
    bne  sum_loop         ; loop if i != limit
    ; r1 now holds 1+2+3+4 = 10

    ; --- NOP (internally mov r0, r0, but special-cased) ---
    nop
    nop

    ; --- Success: halt ---
done:
    halt

    ; --- Failure path ---
fail:
    halt