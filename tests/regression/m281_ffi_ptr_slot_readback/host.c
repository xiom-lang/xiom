/* m281 FFI-pointer batch lock host: write through a caller-owned pointer and
 * expose a foreign buffer for readback (B-24 / B-20 shapes). */
#include <stdint.h>

void __cdecl poke8(unsigned char* dst, uint64_t v) { *(uint64_t*)dst = v; }

static const unsigned g_buf[4] = {
    0x3F800000u, /* 1.0f */
    0x40000000u, /* 2.0f */
    0x40400000u, /* 3.0f */
    0x40800000u  /* 4.0f */
};

const unsigned* xt_buf_ptr(void) { return g_buf; }

int xt_buf_u32(int i) {
    if (i < 0 || i > 3) return -1;
    return (int)g_buf[i];
}
