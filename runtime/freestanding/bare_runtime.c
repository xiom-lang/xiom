/* XIOM freestanding runtime -- v0.65.0 MCU first wave.
 *
 * Minimal C runtime for a bare-metal XIOM program: no CRT, no host libc, no
 * OS. The generated IR only needs the confined-block guard hooks, a bump
 * allocator for Vec/Str, and the trap/panic path. Link with
 * `-nostdlib -ffreestanding` and the arch startup + linker script in this
 * directory.
 *
 * Contract:
 *   - `__heap_start` / `__heap_end` come from the linker script.
 *   - `xiom_guard_heap_enter/exit` are no-ops: freestanding blocks never
 *     discard an arena, so a Vec created outside a block stays valid.
 *   - allocation never frees (bump); a first-wave hello does not need a
 *     general-purpose heap. HAL packages may replace this file.
 *
 * Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

typedef unsigned long xsize;

extern char __heap_start[];
extern char __heap_end[];

static char *xiom_bump_ptr;

static xsize xiom_align8(xsize n) { return (n + 7u) & ~(xsize)7u; }

static void *xiom_bump_alloc(xsize n) {
    if (n == 0) return (void *)0;
    if (xiom_bump_ptr == (char *)0) xiom_bump_ptr = __heap_start;
    n = xiom_align8(n);
    if (xiom_bump_ptr + n > __heap_end) return (void *)0;
    void *p = xiom_bump_ptr;
    xiom_bump_ptr += n;
    return p;
}

/* -- libc substitutes (the compiler may emit calls even with -fno-builtin) -- */
void *memcpy(void *dst, const void *src, xsize n) {
    unsigned char *d = (unsigned char *)dst;
    const unsigned char *s = (const unsigned char *)src;
    while (n--) *d++ = *s++;
    return dst;
}

void *memmove(void *dst, const void *src, xsize n) {
    unsigned char *d = (unsigned char *)dst;
    const unsigned char *s = (const unsigned char *)src;
    if (d < s) {
        while (n--) *d++ = *s++;
    } else {
        d += n;
        s += n;
        while (n--) *--d = *--s;
    }
    return dst;
}

void *memset(void *dst, int c, xsize n) {
    unsigned char *d = (unsigned char *)dst;
    while (n--) *d++ = (unsigned char)c;
    return dst;
}

xsize strlen(const char *s) {
    xsize n = 0;
    while (s[n]) n++;
    return n;
}

void *malloc(xsize n) { return xiom_bump_alloc(n); }
void free(void *p) { (void)p; }
void *realloc(void *p, xsize n) {
    /* The bump allocator cannot grow in place; callers pass the old size via
       xiom_guard_realloc, so this fallback is a fresh allocation. */
    (void)p;
    return xiom_bump_alloc(n);
}

/* -- XIOM runtime ABI ---------------------------------------------------- */
void xiom_guard_heap_enter(void) {}
void xiom_guard_heap_exit(void) {}
void xiom_guard_page_arm(void) {}
void xiom_guard_page_disarm(void) {}

void *xiom_alloc(long long n) { return xiom_bump_alloc((xsize)n); }
void xiom_free(void *p) { (void)p; }

void *xiom_guard_alloc(long long n) { return xiom_bump_alloc((xsize)n); }
void *xiom_guard_realloc(void *old, long long n, long long oldn) {
    void *p = xiom_bump_alloc((xsize)n);
    if (p && old && oldn > 0) memcpy(p, old, (xsize)oldn);
    return p;
}
void xiom_guard_free(void *p) { (void)p; }

/* No arena in freestanding mode: every buffer already outlives the block. */
void *xiom_guard_copy_out(const void *src, long long n) {
    (void)n;
    return (void *)src;
}
void *xiom_guard_copy_out_if_arena(const void *src, long long copy_len, long long alloc_len) {
    (void)copy_len;
    (void)alloc_len;
    return (void *)src;
}
char *xiom_guard_copy_str(const char *src) { return (char *)src; }

long long xiom_trap_enter(void) { return 0; }
void xiom_trap_leave(void) {}
void xiom_fault_note(long long n) { (void)n; }
void xiom_debugger_break(void) {}
void xiom_set_args(int argc, char **argv) {
    (void)argc;
    (void)argv;
}

/* The panic path: a documented infinite loop. HAL packages may override this
 * with a board-specific reset/UART panic handler. `xiom_panic` is weak so a
 * user program can provide its own. */
__attribute__((weak, noreturn)) void xiom_panic(const char *message) {
    (void)message;
    for (;;) {
    }
}

/* -- Unsafe-block trampoline ABI ---------------------------------------- *
 * The emitter routes an unsafe block through xiom_trampoline_call. On a
 * freestanding target there is no guard page/heap retry: call the block
 * directly and remember its result for the get_result query. */
static long long xiom_tramp_result;

long long xiom_trampoline_call(long long fn, void *ctx) {
    long long (*block)(void *) = (long long (*)(void *))(unsigned long)fn;
    xiom_tramp_result = block(ctx);
    return xiom_tramp_result;
}

long long xiom_trampoline_get_result(void) { return xiom_tramp_result; }
long long xiom_trampoline_was_retried(void) { return 0; }
long long xiom_trampoline_was_returned(void) { return 0; }
void xiom_trampoline_set_allow_retry(long long v) { (void)v; }
void xiom_trampoline_set_returned(void) {}

/* -- ARM EHABI personality stubs ---------------------------------------- *
 * LLVM emits .ARM.exidx unwind tables for the IR even with
 * -fno-unwind-tables; the personality routine is never called (no
 * exceptions in a freestanding C/XIOM program), but the exidx relocation
 * still needs a definition. */
void __aeabi_unwind_cpp_pr0(void) {}
void __aeabi_unwind_cpp_pr1(void) {}
void __aeabi_unwind_cpp_pr2(void) {}

/* -- Cortex-M semihosting console --------------------------------------- *
 * QEMU's microbit machine does not route the nRF51 UART to stdio, so a bare
 * MMIO write is invisible. SYS_WRITEC (0x03) prints one character through
 * the ARM semihosting console (`-semihosting`). */
#if defined(__arm__)
int mcu_putc(unsigned char c) {
    char ch = (char)c;
    register int r0 __asm__("r0") = 0x03;
    register int r1 __asm__("r1") = (int)&ch;
    __asm__ volatile("bkpt 0xAB" : : "r"(r0), "r"(r1));
    return 0;
}
#endif
