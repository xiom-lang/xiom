module m37_ref_mut
// Struct `&T` param mutation regression (docs/COMPILER_BUGS.md): struct-typed
// `&T` params were passed BY VALUE, so `x.v.pop()` inside the callee mutated a
// discarded copy — the caller's Vec length never changed. Fix: plain-struct
// `&T` params pass the ADDRESS (%struct.X*), matching scalar `&T`.

type W = { v: Vec[Int]; }

fn pop_it(x: &W) {
  x.v.pop();
}

fn main() -> Int {
  var w = W{ v: Vec[Int].new(); };
  w.v.push(1);
  w.v.push(2);
  pop_it(&w);
  if w.v.len() != 1 { return 1; }
  if w.v[0] != 1 { return 2; }
  return 0;
}
