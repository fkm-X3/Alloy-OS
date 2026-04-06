; Context switching implementation for x86
; void context_switch(cpu_context* old_ctx, cpu_context* new_ctx)

[BITS 32]

global context_switch

section .text

context_switch:
    ; Arguments:
    ;   [esp+4] = old_ctx (pointer to save current context)
    ;   [esp+8] = new_ctx (pointer to load new context)
    
    mov eax, [esp+4]        ; Get old_ctx pointer
    mov edx, [esp+8]        ; Get new_ctx pointer
    
    ; Save current context to old_ctx
    ; Save general purpose registers
    mov [eax+0],  eax       ; Save EAX (will be overwritten, but that's ok)
    mov [eax+4],  ebx
    mov [eax+8],  ecx
    mov [eax+12], edx
    mov [eax+16], esi
    mov [eax+20], edi
    mov [eax+24], ebp
    mov [eax+28], esp
    
    ; Save return address as EIP
    mov ecx, [esp]          ; Get return address from stack
    mov [eax+32], ecx       ; Save as EIP
    
    ; Save segment registers
    mov cx, cs
    mov [eax+36], ecx       ; CS
    mov cx, ds
    mov [eax+40], ecx       ; DS
    mov cx, es
    mov [eax+44], ecx       ; ES
    mov cx, fs
    mov [eax+48], ecx       ; FS
    mov cx, gs
    mov [eax+52], ecx       ; GS
    mov cx, ss
    mov [eax+56], ecx       ; SS
    
    ; Save EFLAGS
    pushfd
    pop ecx
    mov [eax+60], ecx
    
    ; Load new context from new_ctx
    mov eax, edx            ; eax = new_ctx pointer
    
    ; Restore EFLAGS
    mov ecx, [eax+60]
    push ecx
    popfd
    
    ; Restore segment registers (DS, ES, FS, GS)
    ; Note: We don't restore CS and SS as they're set by the CPU
    mov cx, [eax+40]
    mov ds, cx
    mov cx, [eax+44]
    mov es, cx
    mov cx, [eax+48]
    mov fs, cx
    mov cx, [eax+52]
    mov gs, cx
    
    ; Restore stack pointer first (so we can use stack)
    mov esp, [eax+28]
    
    ; Push new EIP onto stack so 'ret' will jump to it
    mov ecx, [eax+32]
    push ecx
    
    ; Restore general purpose registers
    mov ebx, [eax+4]
    mov ecx, [eax+8]
    mov edx, [eax+12]
    mov esi, [eax+16]
    mov edi, [eax+20]
    mov ebp, [eax+24]
    mov eax, [eax+0]        ; Restore EAX last
    
    ; Jump to new EIP (which we pushed on the stack)
    ret
