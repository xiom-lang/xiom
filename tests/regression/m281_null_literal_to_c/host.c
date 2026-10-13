/* m281 B-23 host: report the bits of a pointer argument, and hand back a
 * NULL that ORIGINATED in C. */
#include <stdint.h>

int64_t __cdecl xs_ptr_val(void* p) { return (int64_t)(intptr_t)p; }
void*   __cdecl xs_null(void) { return 0; }
