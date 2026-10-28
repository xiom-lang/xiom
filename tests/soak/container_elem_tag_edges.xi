module container_elem_tag_edges
use xiom.io;
type R2 = { a: Vec[UInt8]; b: Vec[UInt8]; name: Str; }
type R = { id: Int; xs: Vec[UInt8]; }
fn main() -> Int {
  // 1: shadowed name -- the second (Int) binding must not inherit R's tag.
  {
    var rs = Vec[R].new();
    var xs = Vec[UInt8].with_len(256);
    var r = R{ id: 1; xs: xs };
    rs.push(r);
    if rs[0].id != 1 { return 1; }
  }
  {
    var rs = Vec[Int].new();
    rs.push(7);
    rs.push(8);
    if rs[0] != 7 { return 2; }
  }
  // 2: push inside an if, aggregate declared in the same arm.
  var c2 = Vec[R].new();
  if true {
    var x2 = Vec[UInt8].with_len(256);
    var r2 = R{ id: 2; xs: x2 };
    c2.push(r2);
  }
  if c2.len() != 1 { return 3; }
  // 3: aggregate declared OUTSIDE the loop, pushed inside -> tag must be OFF.
  var c3 = Vec[R].new();
  var r3 = R{ id: 3; xs: Vec[UInt8].with_len(256) };
  var k = 0;
  while k < 2 { c3.push(r3); k = k + 1; }
  if c3.len() != 2 { return 4; }
  // 4: with_capacity container, element drop ON.
  var c4 = Vec[R].with_capacity(4);
  var x4 = Vec[UInt8].with_len(256);
  var r4 = R{ id: 4; xs: x4 };
  c4.push(r4);
  c4[0].id = 44;
  if c4[0].id != 44 { return 5; }
  // 5: element with TWO Vec fields and a Str field.
  var c5 = Vec[R2].new();
  var ab = Vec[UInt8].with_len(256);
  var bb = Vec[UInt8].with_len(256);
  var r5 = R2{ a: ab; b: bb; name: "rec" };
  c5.push(r5);
  if c5[0].a.len() != 256 || c5[0].b.len() != 256 { return 6; }
  if c5[0].name != "rec" { return 7; }
  io.println("tag edges ok");
  return 0;
}
