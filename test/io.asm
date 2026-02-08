; ============================================================================
; io.asm — Stack-frame peek/poke routines
;
; Provides: store_frame, load_frame
; Uses peek/poke instructions to read/write SP-relative stack slots.
;
; Exercises: peek/poke (P-type), subsp/addsp with immediates (S-type),
;            inc/dec virtual instructions, cross-module linking.
; ============================================================================

; --- Return point (defined in main.asm) -------------------------------------
.import io_ret

; --- Public API -------------------------------------------------------------
.export store_frame
.export load_frame

; ============================================================================
; store_frame — allocate a 4-byte frame and write r1, r2 into it
;
;   Input:  r1 = first value, r2 = second value
;   Output: (stack frame left allocated with the two values)
; ============================================================================
store_frame:
    subsp 4                   ; allocate 4 bytes on the stack
    poke r1, 0x00             ; MEM[SP + 0] = r1
    poke r2, 0x02             ; MEM[SP + 2] = r2
    br   io_ret

; ============================================================================
; load_frame — read two values from the current stack frame into r1, r2
;              and deallocate it
;
;   Output: r1 = first value, r2 = second value
; ============================================================================
load_frame:
    peek r1, 0x00             ; r1 = MEM[SP + 0]
    peek r2, 0x02             ; r2 = MEM[SP + 2]
    addsp 4                   ; deallocate 4 bytes
    br   io_ret
