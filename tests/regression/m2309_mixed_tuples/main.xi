// REL-20261015-2309-packages lock: mixed-element tuples. A tuple carrying
// Vec/Str/Int and a 4-field mixed tuple must compile, link, and read every
// element (the v0.64.5 failure was `unknown type Tuple__Vec[UInt8]__Str__Int`
// plus garbage/AV). The helper is deliberately NOT named `recv`: a top-level
// XIOM fn with a libc/Winsock name collides at link time (separate defect).
module m2309_mixed_tuples

use xiom.io;

fn frame_bytes() -> (Vec[UInt8], Str, Int) {
  var b = Vec[UInt8].new();
  b.push(65);
  b.push(66);
  b.push(67);
  return (b, "abc", 42);
}

fn four() -> (Int, Str, Int, Str) {
  return (1, "two", 3, "four");
}

fn main() -> Int {
  let t = frame_bytes();
  if t.0.len() != 3 { return 1; }
  if t.0[0] != 65 || t.0[2] != 67 { return 2; }
  if t.1 != "abc" { return 3; }
  if t.2 != 42 { return 4; }
  let f = four();
  if f.0 != 1 { return 5; }
  if f.1 != "two" { return 6; }
  if f.2 != 3 { return 7; }
  if f.3 != "four" { return 8; }
  io.println("mixed tuples ok " + t.1 + " " + f.3);
  return 0;
}
