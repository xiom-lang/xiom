// m152 (packages relay): plain-local calls to `&mut`/pointer out-params must
// write through. v0.61.3 materialized a temp COPY (BUG 31's fallback path),
// so every write through the callee was lost -- and a `&mut Struct` variant
// corrupted memory. The lock counts failures and exits 0 when all writes
// propagate (explicit `&mut` forms included, which must stay correct).
module m152_mut_write_through;

use xiom.convert;
use xiom.io;

type Bag = { v: Int; vs: Vec[Int]; }

fn set_one(out: &mut Int) { *out = 7; }

fn split2(v: Int, a: &mut Int, b: &mut Int) {
  *a = v / 2;
  *b = v - (v / 2);
}

fn bump(out: &mut Int) { *out = *out + 1; }

fn bag_push(b: &mut Bag, x: Int) { b.vs.push(x); }

fn main() -> Int {
  var bad = 0;

  var x = 0;
  set_one(x);
  if x != 7 { bad = bad + 1; }

  var a = 0;
  var b = 0;
  split2(10, a, b);
  if a != 5 { bad = bad + 1; }
  if b != 5 { bad = bad + 1; }

  var c = 41;
  bump(c);
  if c != 42 { bad = bad + 1; }

  // Struct out-param: field state and the contained Vec must both persist
  // through the plain-local call (the corruption variant).
  var bag = Bag{ v: 5; vs: Vec[Int].new(); };
  bag_push(bag, 7);
  if bag.v != 5 { bad = bad + 1; }
  if bag.vs.len() != 1 { bad = bad + 1; }
  if bag.vs[0] != 7 { bad = bad + 1; }

  // Explicit `&mut` must stay correct.
  var e = 0;
  set_one(&mut e);
  if e != 7 { bad = bad + 1; }

  io.println("bad=" + int_to_string(bad));
  return bad;
}
