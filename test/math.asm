; math.asm — small utility routines
.import return_here

.export add_values

add_values:
    ; Add r1 + r2, store result in r1
    add r1, r2

    ; Return to caller
    br  return_here
