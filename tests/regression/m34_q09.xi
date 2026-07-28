// M34-Q09: Contract with pointer — contract-protected pointer deref chain
fn deref_ptr(ptr: *Int) -> Int
  ensures: result >= 0
{
  var v: Int;
  unsafe { v = *ptr; }
  return v;
}
fn double_ptr(ptr: *Int) -> Int
  ensures: result == deref_ptr(ptr) * 2
{
  return deref_ptr(ptr) * 2;
}
fn main() -> Int {
  var x: Int = 42;
  var p: *Int;
  unsafe { p = &x as *Int; }
  var r: Int = double_ptr(p);
  var d: Int = deref_ptr(p);
  if r == 84 && d == 42 { return 0; }
  return 1;
}
