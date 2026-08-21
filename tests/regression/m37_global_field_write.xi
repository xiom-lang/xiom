module m37_global_field_write
// BUG 2 regression: module-global struct FIELD writes must persist.
// `var g: W = W{ v: 0; };` at module scope + `g.v = 5;` inside a fn --
// the write went through compile_lvalue which had no module-global base
// branch, so the store was silently dropped (read back 0).

use xiom.io;

type W = { v: Int; w: Int; }

var g: W = W{ v: 0; w: 0; };

fn _set() {
  g.v = 5;
  g.w = 7;
}

fn main() -> Int {
  _set();
  if g.v != 5 { io.println("fail: g.v = " + g.v); return 1; }
  if g.w != 7 { io.println("fail: g.w = " + g.w); return 2; }
  return 0;
}
