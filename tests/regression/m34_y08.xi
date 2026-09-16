// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Y08: enum with payload matched + compound assign + impl + generic + module
type Accum = { total: Int; count: Int; }
enum Cmd { Add(n: Int), Mul(n: Int), Reset }
fn mul_helper(x: Int, y: Int) -> Int { return x * y; }
fn execute[T](a: Accum, cmd: Cmd) -> Int
  requires: a.total >= 0
  ensures: result >= 0
{
  var t = a.total;
  match cmd {
    Add(n) => { t = t + n; }
    Mul(n) => { t = mul_helper(t, n); }
    Reset => { t = 0; t = t + a.count; }
  }
  return t;
}
interface AccumOps { fn add(self, n: Int) -> Int; fn mul(self, n: Int) -> Int; }
impl AccumOps for Accum {
  fn add(self, n: Int) -> Int { return self.total + n; }
  fn mul(self, n: Int) -> Int { return self.total * n; }
}
module acc {
  pub fn run(a: Accum, c: Cmd) -> Int { return execute(a, c); }
  pub fn via_add(a: Accum, n: Int) -> Int { return a.add(n); }
  pub fn via_mul(a: Accum, n: Int) -> Int { return a.mul(n); }
}
use acc.run;
use acc.via_add;
use acc.via_mul;
fn main() -> Int {
  var a = Accum{ total: 10; count: 2; };
  var r1 = run(a, Cmd.Add(5));
  if r1 == 15 { return 0; }
  return 1;
}
