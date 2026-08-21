// M32-T09: String comparison -- == and != operators
fn main() -> Int {
  var a: Str = "apple";
  var b: Str = "apple";
  var c: Str = "orange";
  if a == b && a != c && b != c { return 0; }
  return 1;
}
