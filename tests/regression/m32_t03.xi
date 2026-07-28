// M32-T03: String concat — + operator
fn main() -> Int {
  var s: Str = "hello";
  var t: Str = " world";
  var u: Str = s + t;
  if u == "hello world" { return 0; }
  return 1;
}
