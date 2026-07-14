// E2E regression: cross-module use of a stdlib module whose functions call C
// externs (math -> fabs/floor). Locks in cross-module extern-declare injection
// and libc/libm wrapper handling. Returns 0 on success.
module e2e_cross_math
use xiom.math;

fn main() -> Int {
  if math.abs_int(-5) == 5 && math.max_int(2, 7) == 7 {
    return 0;
  }
  return 1;
}
