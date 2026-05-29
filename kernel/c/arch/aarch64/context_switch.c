#include "boot/types.h"
#include "../context.h"

void context_switch(cpu_context* old_ctx, cpu_context* new_ctx) {
    if (old_ctx) {
        asm volatile("mov %0, sp" : "=r"(old_ctx->esp));
    }

    if (new_ctx) {
        asm volatile("mov sp, %0" : : "r"(new_ctx->esp));
    }
}