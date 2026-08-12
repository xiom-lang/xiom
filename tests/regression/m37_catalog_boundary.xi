module m37_catalog_boundary
// BUG 22/23 regression battery: catalog-boundary type fidelity.
// 23.1 cross-module returned Vec[Float64] element reads (raw-bit garbage)
// 23.2 nested Vec[Vec[Int]] matrix reads (inner Vecs truncated to 8 bytes)
// 22.4 cross-module match on Option[Int] payload
// 23.7 cross-module (Bool, Bool) tuple fields (misregistered Int__Int)
// 22.3 cross-module 3-tuple .1/.2 field access
// 23.8 catalog &Vec[T] param mutation (silent no-op — by-value ABI)
// 23.9 unary minus on a catalog-returned float

use m37_catmod;

fn main() -> Int {
  // 23.9
  var f = m37_catmod.mk_f(2.0);
  var neg = -f;
  if neg != -3.0 { return 1; }
  // 23.1
  var v = m37_catmod.mk_vecf();
  if v[0] != 1.5 { return 2; }
  if v[1] != 2.5 { return 3; }
  // 23.2
  var m = m37_catmod.mk_matrix();
  if m[0][1] != 2 { return 4; }
  if m[1][0] != 3 { return 5; }
  var row = m[0];
  if row.len() != 2 { return 6; }
  // 22.4
  var o = m37_catmod.mk_opt();
  var payload = 0;
  match o {
    Some(v) => payload = v,
    _ => payload = -1,
  }
  if payload != 5 { return 7; }
  // 22.4 float payload through match
  var of = m37_catmod.mk_optf();
  var fp = 0.0;
  match of {
    Some(d) => fp = d,
    _ => fp = -1.0,
  }
  if fp != 2.5 { return 8; }
  // 23.7
  var t = m37_catmod.mk_bb(true, false);
  if !t.0 { return 9; }
  if t.1 { return 10; }
  // 22.3
  var t3 = m37_catmod.mk_t3(1, 2, 3);
  var x = 240 * t3.1;
  if x != 480 { return 11; }
  if t3.2 != 3 { return 12; }
  if t3.0 != 1 { return 13; }
  // 23.8
  var vv = Vec[Int].new();
  vv.push(1); vv.push(2);
  m37_catmod.fill_ints(&vv);
  if vv.len() != 4 { return 14; }
  if vv[2] != 7 { return 15; }
  m37_catmod.fill_ints_mut(&mut vv);
  if vv.len() != 5 { return 16; }
  if vv[4] != 9 { return 17; }
  return 0;
}
