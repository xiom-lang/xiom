// M32-G08: Generic in return type
fn id[T](x: T) -> T { return x; }
fn check_int() -> Int {
  var a = id(100);
  var b = id(200);
  if a != 100 { return 1; }
  if b != 200 { return 2; }
  return 0;
}
fn main() -> Int {
  return check_int();
}
