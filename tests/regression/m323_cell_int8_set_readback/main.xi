// m323 (REL-20261014-2123-stdlib): Cell[Int8].set() followed by get() read
// pointer-shaped garbage on the v0.64.5 candidate (compile rc 0, run rc 2 on
// the filed witness; rc 0 on the v0.64.4 archive). Root cause: the call-site
// generic fallback paired every explicit ARG with fd.params[i] WITHOUT
// skipping the receiver `self` slot, so the VALUE argument was typed from the
// self param (Named "Self" -> infer_llvm_type(arg) = the arg's OWN LLVM
// type). A narrow arg (`127 as Int8`) stayed i8 while the mono def takes the
// widened i64 payload slot; the callee stored the register's garbage high
// bits and get() loaded an i64 of pointer-shaped garbage.
//
// Second shape (same class, no stdlib): a user generic struct method
// `put[T](&mut self, x: T)` with a narrow value arg went through the same
// shifted pairing (self<-x, value param never consulted).
module m323_cell_int8_set_readback

use xiom.cell;

type Wrap[T] = { v: T; }

fn Wrap.put[T](&mut self, x: T) {
  self.v = x;
}

fn Wrap.take[T](self) -> T {
  return self.v;
}

fn main() -> Int {
  // Filed witness: Cell[Int8] set/get readback.
  var c8 = cell.Cell.new(100 as Int8);
  if c8.get() != 100 as Int8 { return 1; }
  c8.set(127 as Int8);
  if c8.get() != 127 as Int8 { return 2; }
  c8.set(42 as Int8);
  if c8.get() != 42 as Int8 { return 3; }

  // Narrow siblings through the same method-call funnel.
  var c16 = cell.Cell.new(30000 as Int16);
  if c16.get() != 30000 as Int16 { return 4; }
  c16.set(12345 as Int16);
  if c16.get() != 12345 as Int16 { return 5; }

  var c32 = cell.Cell.new(1000000 as Int32);
  if c32.get() != 1000000 as Int32 { return 6; }
  c32.set(7654321 as Int32);
  if c32.get() != 7654321 as Int32 { return 7; }

  // Second shape: user generic struct method with a narrow payload arg.
  var w8 = Wrap[Int8]{ v: 1 as Int8 };
  w8.put(99 as Int8);
  if w8.take() != 99 as Int8 { return 8; }
  var w16 = Wrap[Int16]{ v: 2 as Int16 };
  w16.put(300 as Int16);
  if w16.take() != 300 as Int16 { return 9; }

  return 0;
}
