// M35-M25: integer log base 2 (floor)
fn log2_int(n: Int) -> Int {
  if n <= 0 { return 0; }
  var result: Int = 0;
  var x = n;
  while x > 1 {
    x = x / 2;
    result = result + 1;
  }
  return result;
}
fn main() -> Int {
  if log2_int(1) == 0 && log2_int(2) == 1 && log2_int(3) == 1 && log2_int(8) == 3 && log2_int(1024) == 10 && log2_int(100) == 6 { return 0; }
  return 1;
}
