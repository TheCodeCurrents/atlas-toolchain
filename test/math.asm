; ============================================================================
; math.asm — Arithmetic library routines
;
; Provides: multiply, divide, abs_value
; Each routine receives arguments in r1/r2 and returns by branching to
; an exported return-point label in the caller.
;
; Exercises: loops, conditional branches, shifts, register moves, XOR/NEG,
;            stack push/pop for register saving, and cross-module linking.
; ============================================================================

; --- Return points (defined in main.asm) ------------------------------------
.import mul_ret
.import div_ret
.import abs_ret

; --- Public API -------------------------------------------------------------
.export multiply
.export divide
.export abs_value

; ============================================================================
; multiply — unsigned 8-bit multiply via shift-and-add
;
;   Input:  r1 = multiplicand, r2 = multiplier
;   Output: r1 = r1 * r2  (low 8 bits)
;   Clobbers: r2, r3, r4
; ============================================================================
multiply:
    ; Save r5 on the stack (used as accumulator)
    push r5

    ldi  r3, 0x00             ; r3 = accumulator
    ldi  r4, 0x01             ; r4 = bit mask (1)

mul_loop:
    ; If multiplier (r2) is zero, we're done
    ldi  r5, 0x00
    cmp  r2, r5
    beq  mul_done

    ; Test lowest bit of r2: r5 = r2 AND 1
    mov  r5, r2
    and  r5, r4               ; r5 = r2 & 0x01
    ldi  r6, 0x00
    cmp  r5, r6
    beq  mul_skip_add

    ; Bit is set — add current multiplicand to accumulator
    add  r3, r1

mul_skip_add:
    ; Shift multiplicand left by 1 (double it)
    mov  r5, r1
    add  r1, r5               ; r1 = r1 * 2 (shift left by adding to self)

    ; Shift multiplier right by 1  (shr rd, rs → rd = rs >> 1)
    shr  r2, r2

    br   mul_loop

mul_done:
    ; Move result to r1
    mov  r1, r3
    pop  r5                   ; restore saved register
    br   mul_ret

; ============================================================================
; divide — unsigned 8-bit division via repeated subtraction
;
;   Input:  r1 = dividend, r2 = divisor
;   Output: r1 = quotient, r2 = remainder
;   Clobbers: r3, r5
; ============================================================================
divide:
    push r5

    ldi  r3, 0x00             ; r3 = quotient

    ; Check for divide-by-zero
    ldi  r5, 0x00
    cmp  r2, r5
    beq  div_by_zero

div_loop:
    ; If dividend < divisor, we're done
    cmp  r1, r2
    bcc  div_done              ; unsigned less-than: carry clear

    ; Subtract divisor from dividend
    sub  r1, r2
    addi r3, 0x01              ; increment quotient
    br   div_loop

div_done:
    ; r1 is now the remainder, move quotient to proper place
    mov  r2, r1                ; r2 = remainder
    mov  r1, r3                ; r1 = quotient
    pop  r5
    br   div_ret

div_by_zero:
    ; Return 0xFF as error sentinel
    ldi  r1, 0xFF
    ldi  r2, 0xFF
    pop  r5
    br   div_ret

; ============================================================================
; abs_value — absolute value (two's complement)
;
;   Input:  r1 = signed 8-bit value
;   Output: r1 = |r1|
;   Clobbers: r5
; ============================================================================
abs_value:
    push r5

    ; Check sign bit (bit 7): if r1 AND 0x80 != 0, it's negative
    ldi  r5, 0x80
    mov  r3, r1
    and  r3, r5
    ldi  r5, 0x00
    cmp  r3, r5
    beq  abs_done              ; already positive

    ; Negate: r1 = ~r1 + 1  (two's complement)
    neg  r1, r1

abs_done:
    pop  r5
    br   abs_ret
