// M33-U01: Unsafe block basic — assign and read in unsafe
fn main() -> Int {
  var x: Int = 0;
  unsafe { x = 42; }
  if x != 42 { return 1; }
  return 0;
}
