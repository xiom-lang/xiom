// REL-20261028: nested-struct element-drop safety shapes. Nested Vec fields
// are freed only for owned push-only containers; nested reads/aliases and
// duplicate pushes stay leak-only.
module nested_struct_drops

use xiom.io;

type I1 = { data: Vec[UInt8]; }
type I2 = { data: Vec[UInt8]; }
type O = { a: I1; b: I2; id: Int; }

fn main() -> Int {
  // 1: named aggregate with two nested Vecs; scalar element read.
  var os = Vec[O].new();
  var x = Vec[UInt8].with_len(1024);
  var y = Vec[UInt8].with_len(1024);
  var o = O{ a: I1{ data: x }; b: I2{ data: y }; id: 5 };
  os.push(o);
  if os[0].id != 5 { return 1; }

  // 2: nested Vec-field read -> tag off (leak, no crash).
  var ns = Vec[O].new();
  var nx = Vec[UInt8].with_len(1024);
  var ny = Vec[UInt8].with_len(1024);
  var no = O{ a: I1{ data: nx }; b: I2{ data: ny }; id: 6 };
  ns.push(no);
  if ns[0].a.data.len() != 1024 { return 2; }

  // 3: nested Vec field bound/escaped -> tag off.
  var es = Vec[O].new();
  var ex = Vec[UInt8].with_len(1024);
  var ey = Vec[UInt8].with_len(1024);
  var eo = O{ a: I1{ data: ex }; b: I2{ data: ey }; id: 7 };
  es.push(eo);
  let bound = es[0].b.data;
  if bound.len() != 1024 { return 3; }

  // 4: inline nested literal with fresh leaves -> tag on.
  var ils = Vec[O].new();
  ils.push(O{ a: I1{ data: Vec[UInt8].with_len(1024) }; b: I2{ data: Vec[UInt8].with_len(1024) }; id: 8 });
  if ils[0].id != 8 { return 4; }

  // 5: duplicate push -> tag off.
  var ds = Vec[O].new();
  var dx = Vec[UInt8].with_len(1024);
  var dy = Vec[UInt8].with_len(1024);
  var d = O{ a: I1{ data: dx }; b: I2{ data: dy }; id: 9 };
  ds.push(d);
  ds.push(d);
  if ds.len() != 2 { return 5; }

  io.println("nested struct shapes ok");
  return 0;
}
