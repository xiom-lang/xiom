/* m282 host: call a raw XIOM code pointer and return its value. The
 * pointer must be the raw code address, not a closure ENV handle. */
#include <stdint.h>

typedef int64_t (*xiom_i64_fn)(void);

int64_t __cdecl call6(unsigned char* fp) {
    xiom_i64_fn f = (xiom_i64_fn)fp;
    return f();
}
