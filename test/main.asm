; main.asm — entry point, calls into math module
.import add_values

.export main

RESULT_ADDR: .imm 0x80

main:
    ; Set up arguments: r1 = 0x10, r2 = 0x25
    ldi r1, 0x10
    ldi r2, 0x25

    ; Call the add routine (result returned in r1)
    br  add_values

return_here:
    ; Store the result in memory
    ldi r3, RESULT_ADDR
    st  r1, [r3, 0]

    ; Verify: load it back and compare
    ld  r4, [r3, 0]
    cmp r1, r4
    bne fail

    ; Success
    halt

fail:
    halt
