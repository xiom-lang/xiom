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
extern int main(void);

void _start(void) {
    __asm__ volatile(
        "lui sp, %%hi(_estack)\n"
        "addi sp, sp, %%lo(_estack)\n"
        :
        :
        : "sp");

    int rc = main();

    /* QEMU virt sifive_test: (code << 16) | 0x5555 exits. */
    volatile unsigned int *tohost = (volatile unsigned int *)0x00100000u;
    *tohost = (((unsigned int)rc & 0xffffu) << 16) | 0x5555u;

    for (;;) {
    }
}
