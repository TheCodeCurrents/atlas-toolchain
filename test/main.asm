; ============================================================================
; main.asm — Entry point and test harness
;
; Exercises: imports/exports, constants, branches, memory load/store,
;            stack operations, ALU comparisons, conditional branches,
;            and cross-module calls.
; ============================================================================

; --- External routines (defined in math.asm / io.asm) -----------------------
.import multiply          ; r1 = r1 * r2  (returns via mul_ret)
.import divide            ; r1 = r1 / r2, r2 = r1 % r2  (returns via div_ret)
.import abs_value         ; r1 = |r1|  (returns via abs_ret)
.import store_frame       ; allocate frame, poke r1/r2  (returns via io_ret)
.import load_frame        ; peek r1/r2, deallocate frame (returns via io_ret)

; --- Return-point labels the library routines branch back to ----------------
.export mul_ret
.export div_ret
.export abs_ret
.export io_ret

; --- Public entry point -----------------------------------------------------
.export main

; ============================================================================
; Constants
; ============================================================================
STACK_TOP:    .imm 0xF0     ; initial stack pointer
RESULT_ADDR:  .imm 0x80     ; scratch memory for results
TEMP_ADDR:    .imm 0x82     ; secondary scratch
MAGIC:        .imm 0xAA     ; sentinel value for pass/fail
NUM_TESTS:    .imm 0x06     ; total number of test cases

; ============================================================================
; Code
; ============================================================================
main:
    ; ------ Initialise the stack pointer ------------------------------------
    ldi  sp, STACK_TOP

    ; ------ Test counter in r9 (number of passing tests) --------------------
    ldi  r9, 0x00

; ---- TEST 1: Addition (0x10 + 0x25 = 0x35) --------------------------------
test_add:
    ldi  r1, 0x10
    ldi  r2, 0x25
    add  r1, r2
    ldi  r5, 0x35            ; expected result
    cmp  r1, r5
    bne  test_sub             ; skip increment on failure
    inc  r9                   ; test passed

; ---- TEST 2: Subtraction (0x40 - 0x18 = 0x28) -----------------------------
test_sub:
    ldi  r1, 0x40
    ldi  r2, 0x18
    sub  r1, r2
    ldi  r5, 0x28
    cmp  r1, r5
    bne  test_logic
    inc  r9

; ---- TEST 3: Bitwise logic (0xAA & 0x0F = 0x0A, | 0xF0 = 0xFA) -----------
test_logic:
    ldi  r1, 0xAA
    ldi  r2, 0x0F
    and  r1, r2              ; r1 = 0x0A
    ldi  r3, 0xF0
    or   r1, r3              ; r1 = 0xFA
    ldi  r5, 0xFA
    cmp  r1, r5
    bne  test_shift
    inc  r9

; ---- TEST 4: Shifts (0x01 << 4 = 0x10, >> 2 = 0x04) ----------------------
;   Per ISA: shl rd, rs  →  rd = rs << 1  (always shifts by 1)
;   So we repeat shl 4 times to shift left by 4, then shr 2 times.
test_shift:
    ldi  r1, 0x01
    shl  r1, r1              ; r1 = 0x02
    shl  r1, r1              ; r1 = 0x04
    shl  r1, r1              ; r1 = 0x08
    shl  r1, r1              ; r1 = 0x10
    shr  r1, r1              ; r1 = 0x08
    shr  r1, r1              ; r1 = 0x04
    ldi  r5, 0x04
    cmp  r1, r5
    bne  test_mul
    inc  r9                   ; test passed (uses inc virtual instruction)

; ---- TEST 5: Cross-module multiply (6 * 7 = 42 = 0x2A) --------------------
test_mul:
    ldi  r1, 0x06
    ldi  r2, 0x07
    br   multiply             ; r1 = r1 * r2
mul_ret:
    ldi  r5, 0x2A
    cmp  r1, r5
    bne  test_mem
    inc  r9

; ---- TEST 6: Memory store / load round-trip --------------------------------
test_mem:
    ldi  r1, 0xBE            ; value to store
    ldi  r3, RESULT_ADDR
    st   r1, [r3, 0]         ; MEM[0x80] = 0xBE
    ldi  r1, 0x00            ; clobber r1
    ld   r1, [r3, 0]         ; reload from memory
    ldi  r5, 0xBE
    cmp  r1, r5
    bne  report
    inc  r9

; ============================================================================
; Report: check total passed tests
; ============================================================================
report:
    ; r9 should equal NUM_TESTS if all tests passed
    ldi  r5, NUM_TESTS
    cmp  r9, r5
    bne  fail

pass:
    ; Store the magic sentinel so an external harness can verify
    ldi  r1, MAGIC
    ldi  r3, RESULT_ADDR
    st   r1, [r3, 0]
    halt

fail:
    ; Store 0x00 to indicate failure
    ldi  r1, 0x00
    ldi  r3, RESULT_ADDR
    st   r1, [r3, 0]
    halt

; ============================================================================
; Unused return stubs (for routines we didn't fully exercise above)
; ============================================================================
div_ret:
    nop
    br   report

abs_ret:
    nop
    br   report

io_ret:
    nop
    br   report
