// M35-D06: Priority queue — sorted insert pattern
fn main() -> Int {
  var a: Int = 30;
  var b: Int = 20;
  var c: Int = 10;
  if a < b { return 1; }
  if b < c { return 2; }
  if a < c { return 3; }
  return 0;
}
