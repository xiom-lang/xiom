/* m263 C host: callback reception + a Float64-returning function whose
 * address the XIOM side casts to a fn-pointer (1904). */
#include <stdint.h>

#if defined(_WIN32)
#define XT_EXPORT __declspec(dllexport)
#else
#define XT_EXPORT
#endif

XT_EXPORT int32_t xt_cb_int(int32_t (*cb)(int32_t), int32_t x) {
  return cb(x);
}

XT_EXPORT int32_t xt_cb_int_fn(int32_t (*cb)(int32_t), int32_t x) {
  return cb(x);
}

XT_EXPORT double xt_fp_pi(void) {
  return 3.5;
}

XT_EXPORT int64_t xt_fp_pi_addr(void) {
  return (int64_t)&xt_fp_pi;
}
