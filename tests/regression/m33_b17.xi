// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-B17: Array borrow -- struct-as-tuple passed by &, read fields by index helper
type Triple = { a: Int; b: Int; c: Int; }
fn get_elem(t: &Triple, idx: Int) -> Int {
  if idx == 0 { return t.a; }
  if idx == 1 { return t.b; }
  return t.c;
}
fn main() -> Int {
  var arr = Triple{ a: 10; b: 20; c: 30; };
  var v0 = get_elem(&arr, 0);
  var v1 = get_elem(&arr, 1);
  var v2 = get_elem(&arr, 2);
  if v0 == 10 && v1 == 20 && v2 == 30 && arr.a == 10 { return 0; }
  return 1;
}
