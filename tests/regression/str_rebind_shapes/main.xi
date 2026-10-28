// REL-20261028: Str rebind safety shapes. A replaced owned buffer is freed;
// literals/aliases are never freed; self-assign does not free; a returned
// builder transfers the final value.
module str_rebind_shapes

use xiom.io;
use xiom.convert;

fn build(n: Int) -> Str {
  var s = "";
  var i = 0;
  while i < n {
    s = s + "0123456789";
    i = i + 1;
  }
  return s;
}

fn fresh_then_rebind() -> Int {
  var s = convert.int_to_string(7);
  s = s + "x";
  if s != "7x" { return 2; }
  return 0;
}

fn literal_then_rebind() -> Int {
  var s = "";
  s = s + "a";
  s = s + "b";
  if s != "ab" { return 3; }
  return 0;
}

fn fresh_then_literal() -> Int {
  var s = "";
  s = s + "owned";
  s = "alias";
  if s != "alias" { return 4; }
  return 0;
}

fn self_assign() -> Int {
  var s = "";
  s = s + "keep";
  s = s;
  if s != "keep" { return 5; }
  return 0;
}

fn main() -> Int {
  let b = build(5000);
  if b.len() != 50000 { return 1; }
  if fresh_then_rebind() != 0 { return 6; }
  if literal_then_rebind() != 0 { return 7; }
  if fresh_then_literal() != 0 { return 8; }
  if self_assign() != 0 { return 9; }
  io.println("str rebind shapes ok " + convert.int_to_string(b.len()));
  return 0;
}
