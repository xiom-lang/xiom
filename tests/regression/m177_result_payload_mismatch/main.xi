// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m177 lock (packages relay, consul): a Result[Bool, Str] must NOT satisfy a
// declared Result[Int, Str]. Generic args used to treat Bool as numeric, so
// the mismatch compiled and `.unwrap()` read the Bool payload as Int garbage.
fn make() -> Result[Int, Str] {
  var r: Result[Bool, Str] = Ok(true);
  return r;
}

fn main() -> Int {
  return 0;
}
