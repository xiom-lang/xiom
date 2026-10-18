// m370 (REL-20261018-1546-stdlib): callee reads corrupted fields from
// &Vec[GenericStruct] (silent). Expected: rc 0, `plain=0`.
// Pre-fix (v0.64.7): compile rc 0, run rc 1, `plain=111`.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module p_vec_byref_generic_struct

use xiom.io;
use xiom.convert;

type Box[T] = { items: Vec[T]; closed: Bool; cap: Int; }

fn inspect(bs: &Vec[Box[Int]]) -> Int {
  if bs.len() == 0 { return -1; }
  let b = bs[0];
  var bad = 0;
  if b.items.len() != 0 { bad = bad + 1; }
  if b.closed != true { bad = bad + 10; }
  if b.cap != 7 { bad = bad + 100; }
  return bad;
}

fn main() -> Int {
  var v: Vec[Box[Int]] = Vec[Box[Int]].new();
  var b = Box[Int]{ items: Vec[Int].new(); closed: true; cap: 7; };
  v.push(b);

  // Caller-side controls: these read correctly on the affected pins.
  let m = v[0];
  if m.items.len() != 0 { return 2; }
  if m.closed != true { return 3; }
  if m.cap != 7 { return 4; }
  if v[0].items.len() != 0 { return 5; }
  if v[0].closed != true { return 6; }
  if v[0].cap != 7 { return 7; }

  let bad = inspect(&v);
  io.println("plain=" + convert.int_to_string(bad));
  if bad != 0 { return 1; }
  return 0;
}
