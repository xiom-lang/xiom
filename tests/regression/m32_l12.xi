// M32-L12: If-as-expression -- var x = if cond { a } else { b }
fn main() -> Int {
  var base: Int = 42;
  var val = if base > 30 { 100 } else { 0 };
  var val2 = if base < 0 { 200 } else { 50 };
  if val == 100 && val2 == 50 { return 0; }
  return 1;
}
