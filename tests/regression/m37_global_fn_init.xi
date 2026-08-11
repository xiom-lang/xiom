module m37_global_fn_init
// BUG 3 regression: module-global `var` initializers that CALL functions
// must produce the computed value, not a silent zero. Lowered to a
// @llvm.global_ctors startup initializer.

use xiom.io;

fn _mk(n: Int) -> Int { return n * 10; }
var G_TEN: Int = _mk(1);
var G_TWENTY: Int = _mk(2);

fn main() -> Int {
  if G_TEN != 10 { io.println("fail: G_TEN = " + G_TEN); return 1; }
  if G_TWENTY != 20 { io.println("fail: G_TWENTY = " + G_TWENTY); return 2; }
  return 0;
}
