// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_deep_expr_015
type A = { val: Int; }
  type B = { a: A; }
  type C = { b: B; }
  type D = { c: C; }
  type E = { d: D; }
  type F = { e: E; }

  fn get_val(f: F) -> Int {
    return f.e.d.c.b.a.val;
  }

  pub fn run() -> Int {
    var x: F = { e: { d: { c: { b: { a: { val: 99; }; }; }; }; }; };
    if get_val(x) == 99 { return 0; }
    return 1;
  }
use m21_deep_expr_015.run;
fn main() -> Int { return run(); }
