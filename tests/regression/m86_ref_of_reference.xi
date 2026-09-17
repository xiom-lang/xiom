// m86 (R43): `&v` where v already holds a reference (`&T`) denotes the SAME
// reference (`&*v == v`), not a double-address. Pre-fix this hard-errored
// C001 ("cannot take a reference to 'v': it is already a reference"),
// breaking the stdlib's `var v = b; ... &v` pattern
// (x25519_keypair -> _bigint_to_le, BUG 26 #1 family).
//
// Pins the reference-binding semantics around it:
//   - `&v` on a ref binding reads through to the original,
//   - a FIELD write through the alias mutates the original,
//   - a plain reassignment rebinds the local without touching the original.
module m86_ref_of_reference

type Big = { x: Int; }

fn take_ref(b: &Big) -> Int { return b.x; }

fn alias_and_rebind(b: &Big) -> Int {
  var v = b;
  let first = take_ref(&v);
  v.x = 7;
  let second = take_ref(&v);
  v = Big{ x: 9 };
  let third = take_ref(&v);
  return first * 10000 + second * 100 + third;
}

fn vec_ref_len(v: &Vec[UInt8]) -> Int { return v.len(); }

fn vec_indirect(values: &Vec[UInt8]) -> Int {
  var v = values;
  return vec_ref_len(&v);
}

fn main() -> Int {
  var a = Big{ x: 5 };
  let r = alias_and_rebind(&a);
  if r != 50709 { return 1; }
  if a.x != 7 { return 2; }
  var vals = Vec[UInt8].new();
  vals.push(1);
  vals.push(2);
  vals.push(3);
  if vec_indirect(&vals) != 3 { return 3; }
  return 0;
}
