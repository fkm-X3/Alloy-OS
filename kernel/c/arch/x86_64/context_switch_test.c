void serial_print(const char* s);
extern void context_switch(void* old_ctx, void* new_ctx);

void context_switch_test(void* old_ctx, void* new_ctx) {
    serial_print("[C] context_switch_test called\n");
    serial_print("[C] Calling ASM context_switch\n");
    context_switch(old_ctx, new_ctx);
    serial_print("[C] Returned from ASM context_switch\n");
}
