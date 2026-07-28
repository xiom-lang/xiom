// M34-H14: Cast in function call — argument cast at call site
fn square(x: Int64) -> Int64 { return x * x; }
fn main() -> Int {
  var a: Int8 = 7;
  var r: Int64 = square(a as Int64);
  if r == 49 as Int64 { return 0; }
  return 1;
}
