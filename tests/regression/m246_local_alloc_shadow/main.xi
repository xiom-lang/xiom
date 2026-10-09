// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m246 (sweep finding B-10): a LOCAL fn-pointer named `alloc` (or `free`)
// called inside a confined unsafe block was hijacked by the compiler's
// allocator builtins -- `alloc(35)` routed to @xiom_guard_alloc and
// returned an arena pointer. Only unshadowed builtins may allocate; a
// local binding wins. (odbc 3-way control: f_alloc/my_alloc were green,
// alloc failed.)
module m246_local_alloc_shadow

fn real_alloc(n: Int) -> Int {
  return n + 7;
}

fn real_free(n: Int) -> Int {
  return n + 1;
}

fn main() -> Int {
  let alloc = real_alloc;
  let free = real_free;
  var r = 0;
  unsafe { r = alloc(35); }
  if r != 42 { return 1; }
  var s = 0;
  unsafe { s = free(41); }
  if s != 42 { return 2; }
  return 0;
}
