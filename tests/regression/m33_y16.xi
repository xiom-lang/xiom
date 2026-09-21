// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Y16: generic container struct + enum operations + match + impl method + contract + module + diff
type Stack = { i0: Int; i1: Int; i2: Int; top: Int; }
enum StackOp { Push(v: Int), Pop, Peek }
fn exec[T](s: Stack, op: StackOp) -> Int
  requires: s.top >= 0
  ensures: result >= 0
{
  match op {
    Push(v) => if s.top < 2 { v } else { s.i0 },
    Pop => if s.top > 0 { s.i0 } else { 0 },
    Peek => s.i0,
  }
}
fn peek_direct(s: Stack) -> Int { return s.i0; }
interface StacksOps { fn top_val(self) -> Int; }
impl StacksOps for Stack {
  fn top_val(self) -> Int { return self.i0; }
}
module container {
  pub fn do_exec(s: Stack, op: StackOp) -> Int { return exec(s, op); }
  pub fn do_peek(s: Stack) -> Int { return peek_direct(s); }
  pub fn via_trait(s: Stack) -> Int { return s.top_val(); }
}
use container.do_exec;
use container.do_peek;
use container.via_trait;
enum Path { Exec, Direct, Trait }
fn access(p: Path, s: Stack, op: StackOp) -> Int {
  match p { Exec => do_exec(s, op), Direct => do_peek(s), Trait => via_trait(s), }
}
fn main() -> Int {
  var s = Stack{ i0: 10; i1: 20; i2: 30; top: 0; };
  var r1 = access(Path.Exec, s, StackOp.Peek);
  var r2 = access(Path.Direct, s, StackOp.Peek);
  var r3 = access(Path.Trait, s, StackOp.Peek);
  if r1 == r2 && r2 == r3 && r1 == 10 { return 0; }
  return 1;
}
