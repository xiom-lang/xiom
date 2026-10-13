// m284 lock (B-21 REL-20261011-2140-bindings): `&mut` of a var initialized
// from a struct-returning call (direct return or Result payload extraction)
// crashed codegen with `use of undefined value '%tmp4'` -- the confined
// block's free-variable walk had no `Expr::MutRef` arm, so `&mut q` never
// captured `q`; the block fn referenced the enclosing fn's driver temp.
// Direct `Pair{...}` init was green (control below).
// Pure XIOM. Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m284_mut_of_struct_return

use xiom.io;

pub type Pair = {
  a: Int;
  b: Int;
}

fn set_pair(p: &mut Pair, x: Int, y: Int) {
  p.a = x;
  p.b = y;
}

fn make_pair() -> Pair {
  return Pair{ a: 3, b: 4 };
}

// Result payload extraction variant (the SDK100 Vulkan loader seam shape).
fn make_result() -> Result[Pair, Str] {
  return Ok(Pair{ a: 1, b: 2 });
}

fn main() -> Int {
  var failed = 0;

  // struct-return init + &mut in a confined block
  var q = make_pair();
  unsafe { set_pair(&mut q, 7, 9); }
  if q.a != 7 || q.b != 9 { failed = failed + 1; }

  // literal-init control
  var lit = Pair{ a: 0, b: 0 };
  unsafe { set_pair(&mut lit, 5, 6); }
  if lit.a != 5 || lit.b != 6 { failed = failed + 1; }

  // Result payload extraction + &mut
  let r = make_result();
  if !r.is_ok {
    failed = failed + 1;
  } else {
    var api = r.value;
    unsafe { set_pair(&mut api, 11, 12); }
    if api.a != 11 || api.b != 12 { failed = failed + 1; }
  }

  io.println("q=" + to_string(q.a) + "," + to_string(q.b)
    + " lit=" + to_string(lit.a) + "," + to_string(lit.b)
    + " failures=" + to_string(failed));
  return failed;
}
