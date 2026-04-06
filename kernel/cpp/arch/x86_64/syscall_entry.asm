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
    ; Syscall number is in EAX (already pushed)
    push edi    ; arg4
    push esi    ; arg3
    push edx    ; arg2
    push ecx    ; arg1
    push ebx    ; arg0
    push eax    ; syscall number
    
    ; Call C++ dispatcher
    call syscall_dispatcher
    
    ; Clean up pushed arguments (6 * 4 = 24 bytes)
    add esp, 24
    
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
