// M32: Result Int8 payload (Ok)
fn main() -> Int {
  var r: Result[Int8, Bool] = Ok(-128 as Int8);
  if r.is_ok() { return 0; }
  return 1;
}
