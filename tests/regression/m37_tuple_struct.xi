module m37_tuple_struct
// BUG 1 regression (docs/COMPILER_BUGS.md): tuple returns containing structs.
// Before the fix, the fn signature used bare element names (Tuple__Big__Big)
// while the body used module-qualified names -- clang rejected the IR ("Cannot
// allocate unsized type") or the tuple slots were truncated to the first i64
// of each struct (garbage Vec pointers at runtime). Also covers the
// copy-from-tuple pattern and 3-element tuples.

type Big = { d: Vec[Int]; neg: Bool; }

fn mk_big(v: Int) -> Big {
  var x = Big{ d: Vec[Int].new(); neg: false; };
  x.d.push(v);
  return x;
}

fn div_mod(a: Big, b: Big) -> (Big, Big) {
  return (mk_big(7), mk_big(3));
}

fn triple(a: Big) -> (Big, Big, Big) {
  return (mk_big(1), mk_big(2), mk_big(3));
}

fn main() -> Int {
  var dm = div_mod(mk_big(5), mk_big(2));
  if dm.0.d[0] != 7 { return 1; }
  if dm.1.d[0] != 3 { return 2; }
  // Copy tuple elements into locals (the stdlib `var q = dm.0;` pattern).
  var q = dm.0;
  if q.d[0] != 7 { return 3; }
  if q.neg != false { return 4; }
  // 3-element tuple of 40-byte structs.
  var tr = triple(mk_big(0));
  if tr.0.d[0] != 1 { return 5; }
  if tr.1.d[0] != 2 { return 6; }
  if tr.2.d[0] != 3 { return 7; }
  return 0;
}
