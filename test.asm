.import mark
.export loop

loop:
    ldi r2, 0x12
    ldi r3, 0x34
    add r2, r3
    br loop
    br mark