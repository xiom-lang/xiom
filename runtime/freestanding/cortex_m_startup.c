/* Cortex-M (thumbv6m/thumbv7em/thumbv8m.main) freestanding startup.
 *
 * Vector table at 0x0, Reset_Handler copies .data / zeroes .bss and calls
 * `main`. On return it exits through ARM semihosting (QEMU `-semihosting`);
 * on hardware without a debugger attached the bkpt is ignored and the CPU
 * stays in the documented trap loop.
 *
 * Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

extern unsigned int _estack;
extern unsigned int __data_load;
extern unsigned int __data_start;
extern unsigned int __data_end;
extern unsigned int __bss_start;
extern unsigned int __bss_end;
extern int main(void);

void Reset_Handler(void);
void Default_Handler(void);

__attribute__((section(".vectors"), used))
void (*const xiom_vector_table[])(void) = {
    (void (*)(void))&_estack,
    Reset_Handler,
    Default_Handler, /* NMI */
    Default_Handler, /* HardFault */
    Default_Handler, /* MemManage */
    Default_Handler, /* BusFault */
    Default_Handler, /* UsageFault */
    0, 0, 0, 0,
    Default_Handler, /* SVCall */
    Default_Handler, /* DebugMon */
    0,
    Default_Handler, /* PendSV */
    Default_Handler, /* SysTick */
};

static void xiom_semihost_exit(int code) {
    (void)code;
    /* SYS_EXIT (0x18), ADP_Stopped_ApplicationExit (0x20026). */
    register int r0 __asm__("r0") = 0x18;
    register int r1 __asm__("r1") = 0x20026;
    __asm__ volatile("bkpt 0xAB" : : "r"(r0), "r"(r1));
}

__attribute__((noreturn)) void Reset_Handler(void) {
    unsigned int *src = &__data_load;
    unsigned int *dst = &__data_start;
    while (dst < &__data_end) {
        *dst++ = *src++;
    }
    dst = &__bss_start;
    while (dst < &__bss_end) {
        *dst++ = 0;
    }
    int rc = main();
    xiom_semihost_exit(rc);
    for (;;) {
    }
}

__attribute__((noreturn)) void Default_Handler(void) {
    for (;;) {
    }
}
