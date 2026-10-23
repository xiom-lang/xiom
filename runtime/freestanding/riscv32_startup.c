/* riscv32 (imc/imac) freestanding startup for QEMU `virt`.
 *
 * `_start` sets sp, calls `main`, then writes the SiFive test-device
 * finisher at 0x100000 so QEMU exits with the program's status. On real
 * hardware, replace this file (or the write target) with the board's
 * boot ROM entry.
 *
 * Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

extern unsigned int _estack;
extern unsigned int __bss_start;
extern unsigned int __bss_end;
extern int main(void);

void _start(void) {
    __asm__ volatile(
        "lui sp, %%hi(_estack)\n"
        "addi sp, sp, %%lo(_estack)\n"
        :
        :
        : "sp");

    /* The recursion-depth guard and the bump allocator live in .bss; zero it
       before main, or the first guard check reads garbage and traps. The
       Cortex-M startup does the same. (.data is RAM-resident in this
       image, so there is no load-address copy.) */
    unsigned int *dst = &__bss_start;
    while (dst < &__bss_end) {
        *dst++ = 0;
    }

    int rc = main();

    /* QEMU virt sifive_test: (code << 16) | 0x5555 exits. */
    volatile unsigned int *tohost = (volatile unsigned int *)0x00100000u;
    *tohost = (((unsigned int)rc & 0xffffu) << 16) | 0x5555u;

    for (;;) {
    }
}
