// M34-Q15: Multi-function contract chain — 8 functions calling each other with contracts
fn f1(x: Int) -> Int
  requires: x >= 0
  ensures: result == x
{ return x; }
fn f2(x: Int) -> Int
  requires: x >= 0
  ensures: result == f1(x) + 1
{ return f1(x) + 1; }
fn f3(x: Int) -> Int
  requires: x >= 0
  ensures: result == f2(x) + 2
{ return f2(x) + 2; }
fn f4(x: Int) -> Int
  requires: x >= 0
  ensures: result > f3(x)
{ return f3(x) * 2 + f1(x); }
fn f5(x: Int) -> Int
  requires: x >= 0
  ensures: result >= f4(x)
{ return f4(x) + f2(x); }
fn f6(x: Int) -> Int
  requires: x >= 0
{ var a = f5(x); var b = f3(x); return a + b; }
fn f7(x: Int) -> Int
  requires: x >= 0
  ensures: result > f6(x)
{ return f6(x) * 2; }
fn f8(x: Int) -> Int
  requires: x >= 0
{ var a = f7(x); var b = f1(x); return a - b; }
fn main() -> Int {
  var r = f8(1);
  if r == 29 { return 0; }
  return 1;
}
