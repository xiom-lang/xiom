// m143 (by-value receiver container mutation): a method declared with an
// explicit by-value `self` that mutates a CONTAINER FIELD (`self.v.push(x)`,
// `self.m.insert(...)`, `self.s.insert(...)`) must use the POINTER receiver
// ABI. With a by-value receiver copy the Vec/Map/Set header update (len/cap,
// realloc'd data pointer) landed only in the callee, so the caller's
// container silently kept its old length: `s.add(1); s.count() == 0`.
// The bare-field (this-based) form and a pure read-only accessor (which must
// KEEP the by-value ABI) are covered too.
module m143_receiver_container_mutation;

use xiom.collections;

pub type S = {
  v: Vec[Int];
  m: Map[Int, Int];
  s: Set[Int];
}

// Explicit by-value `self` with a Vec push on a field.
pub fn S.add(self, x: Int) { self.v.push(x); }

// Explicit by-value `self` with a Map insert on a field.
pub fn S.put(self, k: Int, val: Int) { self.m.insert(k, val); }

// Explicit by-value `self` with a Set insert on a field.
pub fn S.mark(self, x: Int) { self.s.insert(x); }

// Bare-field (this-based) Vec push: `v` is the receiver's field.
pub fn S.bump() { v.push(7); }

// Pure read-only accessor: must keep the by-value ABI.
pub fn S.count(self) -> Int { return self.v.len(); }

fn main() -> Int {
  var s = S{ v: Vec[Int].new(); m: Map[Int, Int].new(); s: Set[Int].new() };

  s.add(1);
  if s.count() != 1 { return 2; }
  s.add(2);
  if s.count() != 2 { return 3; }

  s.put(10, 42);
  if s.m.len() != 1 { return 4; }
  if s.m.get(10).unwrap() != 42 { return 5; }

  s.mark(5);
  if s.s.len() != 1 { return 6; }

  s.bump();
  if s.count() != 3 { return 7; }

  return 0;
}
