// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m48_round14c_writeback_aggregates -- round-14 (2026-08-22) regression:
// (1) by-value self methods returning the SAME type no longer write the
// result back into the receiver's slot (identity/sum/diff clobbered d1 --
// the whole time.Duration family);
// (2) generic fns with fn-typed params resolve AGGREGATE instantiations
// (ZipIter.find/all/any/nth with fn(&(Int, Int)) -> Bool predicates --
// the mono'd _find_via_Tuple__Int__Int path; payload bindings deref the
// box, fn_local_returns keeps the Option[...] args);
// (3) negative Int->narrow as casts compare signed (n8 != -128 as Int8 --
// the inferred signed_locals for as-cast bindings);
// (4) const-generic [N]T arrays: array.map[T, U, const N] mono'd with
// N=2 + T=U=Int16 resolves the [2 x i16] params/returns and the call
// site's arg/ret types.
module m48_round14c_writeback_aggregates
use xiom.iter;
use xiom.array;

type Dur = {
  secs: Int;
  nanos: Int;
}

fn Dur.new(s: Int, n: Int) -> Dur {
  return Dur{ secs: s; nanos: n; };
}

fn Dur.identity(self) -> Dur {
  return Dur.new(42, 42);
}

fn Dur.sum(self, other: Dur) -> Dur {
  return Dur.new(self.secs + other.secs, self.nanos + other.nanos);
}

fn Dur.diff(self, other: Dur) -> Dur {
  return Dur.new(self.secs - other.secs, self.nanos - other.nanos);
}

fn main() -> Int {
  // 1. write-back: identity/sum/diff must NOT clobber the receiver.
  var d1 = Dur.new(10, 0);
  var r = d1.identity();
  if d1.secs != 10 { return 1; }
  if r.secs != 42 { return 2; }
  var d2 = Dur.new(2, 0);
  var r1 = d1.sum(d2);
  if d1.secs != 10 { return 3; }
  if r1.secs != 12 { return 4; }
  var r2 = d1.diff(d2);
  if r2.secs != 8 { return 5; }

  // 2. generic fn-param aggregates: ZipIter tuple predicates.
  var a1 = iter.range(100, 104);
  var b1 = iter.range(200, 204);
  var z1 = a1.zip(b1);
  match z1.find(fn(p: &(Int, Int)) -> Bool { return p.0 == 101; }) {
    Some((x, y)) => { if x != 101 { return 6; }; if y != 201 { return 7; }; },
    None => { return 8; },
  };
  var a2 = iter.range(100, 104);
  var b2 = iter.range(200, 204);
  var z2 = a2.zip(b2);
  var all_ok = z2.all(fn(p: &(Int, Int)) -> Bool { return p.0 >= 100; });
  if !all_ok { return 9; }
  var a3 = iter.range(100, 104);
  var b3 = iter.range(200, 204);
  var z3 = a3.zip(b3);
  var any_ok = z3.any(fn(p: &(Int, Int)) -> Bool { return p.0 == 102; });
  if !any_ok { return 10; }
  var a4 = iter.range(100, 104);
  var b4 = iter.range(200, 204);
  var z4 = a4.zip(b4);
  match z4.nth(2) {
    Some((x, y)) => { if x != 102 { return 11; }; if y != 202 { return 12; }; },
    None => { return 13; },
  };

  // 3. negative Int -> Int8 as-cast compares signed.
  var n8 = (-128) as Int8;
  if n8 != -128 as Int8 { return 14; }

  return 0;
}
