// M32-C14: Chained contracts -- functions calling contracted functions
fn double(x: Int) -> Int
  requires: x >= 0
  ensures: result == 2 * x
{
  return x * 2;
}
fn add_doubles(a: Int, b: Int) -> Int
  requires: a >= 0 && b >= 0
  ensures: result > 0
{
  var da: Int = double(a);
  var db: Int = double(b);
  return da + db;
}
fn main() -> Int {
  var r: Int = add_doubles(3, 4);
  if r == 14 { return 0; }
  return 1;
}
