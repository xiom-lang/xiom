// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Y19: nested types + enum dispatch + struct method + match + contract + module + impl + diff
type Inner = { val: Int; }
type Outer = { inner: Inner; mult: Int; }
enum OpTag { GetInner, GetScale, GetProduct }
fn operate[T](o: Outer, tag: OpTag) -> Int
  requires: o.inner.val >= 0
  requires: o.mult >= 0
  ensures: result >= 0
{
  match tag {
    GetInner => o.inner.val,
    GetScale => o.inner.val * o.mult,
    GetProduct => o.inner.val * o.mult,
  }
}
fn product_direct(o: Outer) -> Int { return o.inner.val * o.mult; }
interface NestedOps { fn compute(self) -> Int; }
impl NestedOps for Outer {
  fn compute(self) -> Int { return self.inner.val * self.mult; }
}
module nest {
  pub fn do_operate(o: Outer, tag: OpTag) -> Int { return operate(o, tag); }
  pub fn do_product(o: Outer) -> Int { return product_direct(o); }
  pub fn via_impl(o: Outer) -> Int { return o.compute(); }
}
use nest.do_operate;
use nest.do_product;
use nest.via_impl;
enum Path { Op, Direct, Impl }
fn calc(p: Path, o: Outer, tag: OpTag) -> Int {
  match p { Op => do_operate(o, tag), Direct => do_product(o), Impl => via_impl(o), }
}
fn main() -> Int {
  var inner = Inner{ val: 4; };
  var outer = Outer{ inner: inner; mult: 3; };
  var r1 = calc(Path.Op, outer, OpTag.GetScale);
  var r2 = calc(Path.Direct, outer, OpTag.GetScale);
  var r3 = calc(Path.Impl, outer, OpTag.GetScale);
  if r1 == r2 && r2 == r3 && r1 == 12 { return 0; }
  return 1;
}
