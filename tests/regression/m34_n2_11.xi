// M34-N2-11: Closure chain — closure calls another closure, multi-capture
fn main() -> Int {
  var a = 10;
  var b = 20;
  var f = |x| a + x;
  var g = |y| b + f(y);
  if g(5) == 35 { return 0; }
  return 1;
}
