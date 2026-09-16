// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N2-13: Borrow-in-borrow pattern -- deep reference chain &T -> &U -> read field
type A = { val: Int; }
type B = { a: A; }
type C = { b: B; }

fn read_a_through_b(b: &B) -> Int { return b.a.val; }
fn read_a_through_c(c: &C) -> Int { return c.b.a.val; }

fn main() -> Int {
  var x = C{ b: B{ a: A{ val: 42; }; }; };
  var r1 = read_a_through_c(&x);
  var r2 = read_a_through_b(&x.b);
  if r1 == 42 && r2 == 42 && x.b.a.val == 42 { return 0; }
  return 1;
}
