// M34-Q16: Contract with extern — extern declarations with contract chain
extern "C" {
  fn xiom_str_len(s: *Int) -> Int;
  fn xiom_str_concat(a: *Int, b: *Int) -> *Int;
}

fn safe_strlen(s: *Int) -> Int
  requires: unsafe { s != (0 as *Int) }
  ensures: result >= 0
{
  return xiom_str_len(s);
}
fn validate_ptr(ptr: *Int) -> Int
  requires: unsafe { ptr != (0 as *Int) }
  ensures: result >= 0
{
  return safe_strlen(ptr);
}
fn main() -> Int {
  var x: Int = 42;
  var sum: Int = 0;
  var p: *Int;
  unsafe { p = &x as *Int; }
  if unsafe { p != (0 as *Int) } { return 0; }
  return 1;
}
