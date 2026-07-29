// M32: Result UInt16 Err type with unwrap_or
fn main() -> Int {
  var r: Result[Bool, UInt16] = Err(50000);
  if r.is_err() { return 0; }
  return 1;
}
