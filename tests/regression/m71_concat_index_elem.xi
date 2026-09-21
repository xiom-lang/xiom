// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m71_concat_index_elem
// R14 regression: `"e[0]=" + e[0]` on a Vec produced by a chained
// `.collect()` emitted `inttoptr` for the indexed Int element (garbage
// string pointer -> AV at the element's value, here 9 -> address 0x9).
// The concat must format indexed elements via xiom_int_to_string. The
// bug needed the single-expression chain shape: splitting the chain into
// a local first hid it (the Vec's element registry existed).

use xiom.iter;
use xiom.io;

fn main() -> Int {
  var e = iter.range(1, 50)
    .filter(fn(x: &Int) -> Bool { return *x % 3 == 0; })
    .map(fn(x: Int) -> Int { return x * x; })
    .take(5)
    .collect();
  var s = "e[0]=" + e[0];
  if s != "e[0]=9" { io.println("got: " + s); return 1; }
  var t = "e[4]=" + e[4];
  if t != "e[4]=225" { io.println("got: " + t); return 2; }
  // Indexed element on the LEFT side of the concat.
  var w = e[1] + "!";
  if w != "36!" { io.println("got: " + w); return 3; }
  // Plain (non-chained) Vec index must keep working.
  var v = Vec[Int].new();
  v.push(7);
  var u = "v[0]=" + v[0];
  if u != "v[0]=7" { io.println("got: " + u); return 4; }
  return 0;
}
