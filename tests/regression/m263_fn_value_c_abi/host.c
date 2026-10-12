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

/* m277 (REL-20261012-1630-bindings B-22): the CAST-CALL setter shape --
 * the XIOM side takes this address, casts it to a fn pointer and passes a
 * fn-typed PARAM as the callback argument (pre-fix the closure ENV
 * crossed; C consumed its first argument as the hidden env -> 0). The
 * int(int,int) twin locks nested fn-type parsing
 * (REL-20261012-1403-packages). */
XT_EXPORT int32_t xt_cb_int_fn_export(int32_t (*cb)(int32_t), int32_t x) {
  return cb(x);
}

XT_EXPORT int64_t xt_cb_int_fn_addr(void) {
  return (int64_t)&xt_cb_int_fn_export;
}

XT_EXPORT int32_t xt_cb2_int_fn(int32_t (*cb)(int32_t, int32_t), int32_t x) {
  return cb(x, x);
}

XT_EXPORT int64_t xt_cb2_int_fn_addr(void) {
  return (int64_t)&xt_cb2_int_fn;
}
