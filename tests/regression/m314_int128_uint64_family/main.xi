// m314 (REL-20261013-2042-engine HIGH + -2042-engine-2 + -2227-engine):
// native 128-bit payloads inside Vec elements / stored structs, plus the
// UInt64 bit-63 family (ordering, div/mod, widening).
//
// Pre-fix reds:
//   * LOUD: `Vec[Int128].push(x)` / `v[i] = x` ran the scalar element store
//     with an i128 SSA operand -> clang abort
//     "'%tmp' defined with type 'i128' but expected 'i64'".
//   * SILENT: a struct with an Int128 field was sized by its naive field
//     sum ({ id: Int128; tag: Int } -> 16) while LLVM lays it out at 32
//     bytes; the Vec slot stride truncated the sibling `tag` field, which
//     read back uninitialized stack garbage.
//   * SILENT: every UInt64 `<`/`<=`/`>`/`>=` lowered to signed icmp and
//     `/`/`%` to sdiv/srem, so 0x8000000000000000 compared as negative;
//     UInt64 -> Int128 widening sign-extended through binary-op/`as`-source
//     expression holes.
//
// Exit codes: distinct per shape (compile-time abort pre-fix on shapes 1-2b;
// silent wrong values 8-22 pre-fix).
module m314_int128_uint64_family

pub type Cell128 = {
  id: Int128;
  tag: Int;
}

pub type UBox128 = {
  tag: Int;
  big: UInt128;
}

fn main() -> Int {
  // Shape 1 (LOUD abort pre-fix): Int128 payload in a Vec element.
  var v: Vec[Int128] = Vec[Int128].new();
  var x: Int128 = 170141183460469231731687303715884105727;
  v.push(x);
  let vr: Int128 = v[0];
  if vr != 170141183460469231731687303715884105727 as Int128 { return 1; }

  // Shape 2 (SILENT sibling corruption pre-fix): 32-byte { i128, i64 }
  // struct stored in a Vec must keep its full slot stride.
  var cellv: Vec[Cell128] = Vec[Cell128].new();
  let c = Cell128 { id: 9223372036854775808 as Int128, tag: 77 };
  cellv.push(c);
  let cr = cellv[0];
  if cr.tag != 77 { return 2; }
  if cr.id != 9223372036854775808 as Int128 { return 3; }

  // Shape 2b: UInt128 payloads in Vec elements and Vec-stored structs.
  var uv: Vec[UInt128] = Vec[UInt128].new();
  var ux: UInt128 = 0x80000000000000000000000000000000 as UInt128;
  uv.push(ux);
  let ur: UInt128 = uv[0];
  if ur != 0x80000000000000000000000000000000 as UInt128 { return 4; }
  var boxv: Vec[UBox128] = Vec[UBox128].new();
  let ub = UBox128 { tag: 9, big: 0x80000000000000000000000000000000 as UInt128 };
  boxv.push(ub);
  let ubr = boxv[0];
  if ubr.tag != 9 { return 5; }
  if ubr.big != 0x80000000000000000000000000000000 as UInt128 { return 6; }

  // Shape 2c: indexed 128-bit slot store (`v[0] = x`).
  var sv: Vec[Int128] = Vec[Int128].new();
  sv.push(1 as Int128);
  sv[0] = -1 as Int128;
  let svr: Int128 = sv[0];
  if svr != -1 as Int128 { return 7; }

  // Shape 3 (SILENT signed ordering pre-fix): all four UInt64 orderings.
  var a: UInt64 = 0x8000000000000000;
  var b: UInt64 = 0x0000000000000001;
  if a < b { return 8; }
  if !(a > b) { return 9; }
  if a <= b { return 10; }
  if !(a >= b) { return 11; }
  if b >= a { return 12; }
  var cmax: UInt64 = 0xFFFFFFFFFFFFFFFF;
  if cmax < a { return 13; }
  if !(cmax > a) { return 14; }

  // Shape 4 (SILENT signed div/mod pre-fix).
  var q: UInt64 = a / 2;
  if q != 0x4000000000000000 { return 15; }
  var r: UInt64 = a % 3;
  if r != 2 { return 16; }
  var q2: UInt64 = cmax / 0x10000;
  if q2 != 0xFFFFFFFFFFFF { return 17; }

  // Shape 5 (SILENT sign-extending widening pre-fix): UInt64 -> Int128
  // through binary-source `as`, mixed i128 operand widening, and a
  // Vec[UInt64] index source.
  let expect: Int128 = 9223372036854775808 as Int128;
  var w1: Int128 = (a + 0) as Int128;
  if w1 != expect { return 18; }
  var w2: Int128 = (0 as Int128) + a;
  if w2 != expect { return 19; }
  var w3: Int128 = (0 as Int128) | a;
  if w3 != expect { return 20; }
  var uv2: Vec[UInt64] = Vec[UInt64].new();
  uv2.push(a);
  var w4: Int128 = uv2[0] as Int128;
  if w4 != expect { return 21; }
  var big2: UInt128 = a as UInt128;
  if big2 != 0x8000000000000000 as UInt128 { return 22; }

  // Shape 6 (control): a stored struct with an Int128 field and a sibling.
  var h = Cell128 { id: 170141183460469231731687303715884105727, tag: 42 };
  if h.tag != 42 { return 23; }
  h.id = -1 as Int128;
  if h.id != -1 as Int128 { return 24; }
  if h.tag != 42 { return 25; }

  return 0;
}
