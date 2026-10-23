/* riscv32 freestanding linker script -- QEMU `virt` (kernel load address
 * 0x80200000; the board reserves 0x80000000 for firmware). Bring-your-own-
 * linker-script: copy and adjust for a real board.
 *
 * Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

ENTRY(_start)

MEMORY
{
    RAM (rwx) : ORIGIN = 0x80200000, LENGTH = 16M
}

SECTIONS
{
    .text :
    {
        KEEP(*(.text.start))
        *(.text*)
        *(.rodata*)
        *(.srodata*)
    } > RAM

    .data :
    {
        __data_start = .;
        *(.data*)
        *(.sdata*)
        __data_end = .;
    } > RAM

    .bss (NOLOAD) :
    {
        __bss_start = .;
        *(.bss*)
        *(.sbss*)
        *(COMMON)
        __bss_end = .;
    } > RAM

    .heap (NOLOAD) :
    {
        . = ALIGN(8);
        __heap_start = .;
        . = . + 0x4000;
        __heap_end = .;
    } > RAM

    _estack = ORIGIN(RAM) + LENGTH(RAM);
}
