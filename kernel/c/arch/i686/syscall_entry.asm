; System call entry stub for INT 0x80
; Parameters passed in registers:
;   EAX = syscall number
;   EBX, ECX, EDX, ESI, EDI = arguments
; Return value in EAX

[BITS 32]

global syscall_entry
extern syscall_dispatcher

section .text

syscall_entry:
    ; Save all registers (we'll need them for context)
    push ebp
    push edi
    push esi
    push edx
    push ecx
    push ebx
    push eax
    
    ; Push syscall number and args for C dispatcher
    push edi    ; arg4
    push esi    ; arg3
    push edx    ; arg2
    push ecx    ; arg1
    push ebx    ; arg0
    push eax    ; syscall number
    
    ; Push pointer to INT 0x80 frame (ss, esp, eflags, cs, eip on original stack)
    ; After 6 arg pushes (24 bytes) + 7 saved regs (28 bytes) = 52 bytes
    lea eax, [esp + 52]
    push eax    ; arg5 = int80_frame pointer
    
    ; Call C dispatcher
    call syscall_dispatcher
    
    ; Clean up pushed arguments (7 * 4 = 28 bytes)
    add esp, 28
    
    ; EAX now contains return value from syscall_dispatcher
    ; Save it temporarily
    mov [esp], eax  ; Overwrite saved EAX with return value
    
    ; Restore all registers except EAX (which has return value)
    pop eax    ; This is the return value
    pop ebx
    pop ecx
    pop edx
    pop esi
    pop edi
    pop ebp
    
    ; Return from interrupt
    iret
