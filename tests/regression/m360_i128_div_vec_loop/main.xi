// m360 (REL-20261016-2331-stdlib): a RUNTIME 128-bit division (`sdiv i128`,
// emitted from `/` or `%` on Int128 with a non-constant divisor) that sits
// inside a function which also contains a vector element-load loop compiled
// rc 0 and AV'd with 0xC0000005 at run time when the function lived in an
// xiom.*-namespace module and was called across the module boundary.
//
// Pre-fix signature: compile rc 0, run 0xC0000005. Isolated legs were green
// (loop alone, division in a foldable function, constant divisors) because
// they let LLVM constant-fold the divide; the vector loop blocked folding,
// exposing the runtime helper's recursive single-limb `__udivti3` path.
//
// The fixture runs the witness shape (expected value 2) plus signed and
// unsigned truncating-division edges. The e2e lock compiles it at
// --opt-level 0 (nothing folds, the inline long division actually runs) and
// at the default level, and pins the emitted IR: no `sdiv`/`srem`/`udiv`/
// `urem` on i128 may survive.
//
// Exit codes: 1 = witness value wrong, 2..11 = edge value wrong.
module m360_i128_div_vec_loop

use xiom.math.p_i128_div_vec_loop_mod;

fn main() -> Int {
  // Witness shape: the loop sums 3000 + 4000 = 7000 and the division returns
  // round(2*7000 + 4000 / (2*4000)) = 2.
  var v = Vec[Int].new();
  v.push(3000);
  v.push(4000);
  var sc = v[1];
  var r = p_i128_div_vec_loop_mod.ta4(&v, sc);
  if r != 2 { return 1; }

  // Signed truncation: quotient rounds toward zero, remainder follows the
  // dividend's sign. Pre-fix these never ran (the AV hit first); they also
  // cover the magnitude negation path of the inline lowering.
  var a: Int128 = 18000 as Int128;
  var b: Int128 = 8000 as Int128;
  if a / b != 2 as Int128 { return 2; }
  if a % b != 2000 as Int128 { return 3; }
  if (0 as Int128 - a) / b != -2 as Int128 { return 4; }
  if (0 as Int128 - a) % b != -2000 as Int128 { return 5; }
  if a / (0 as Int128 - b) != -2 as Int128 { return 6; }
  if a % (0 as Int128 - b) != 2000 as Int128 { return 7; }
  if (0 as Int128 - a) / (0 as Int128 - b) != 2 as Int128 { return 8; }

  // Divisor larger than the dividend.
  if 10 as Int128 / 20 as Int128 != 0 as Int128 { return 9; }
  if 10 as Int128 % 20 as Int128 != 10 as Int128 { return 10; }

  // Unsigned twin.
  var ua: UInt128 = 18000 as UInt128;
  var ub: UInt128 = 8000 as UInt128;
  if ua / ub != 2 as UInt128 { return 11; }
  if ua % ub != 2000 as UInt128 { return 12; }

  return 0;
}
