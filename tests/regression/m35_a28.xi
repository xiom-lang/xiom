// M35-A28: Binary conversion -- verify integer to binary digit extraction correctness
fn count_bits(n: Int) -> Int {
  var x: Int = n;
  var c: Int = 0;
  while x > 0 {
    c = c + (x & 1);
    x = x >> 1;
  }
  return c;
}
fn binary_length(n: Int) -> Int {
  if n == 0 { return 1; }
  var x: Int = n;
  var l: Int = 0;
  while x > 0 {
    l = l + 1;
    x = x >> 1;
  }
  return l;
}
fn main() -> Int {
  if count_bits(0) != 0 { return 1; }
  if count_bits(7) != 3 { return 2; }
  if count_bits(15) != 4 { return 3; }
  if count_bits(256) != 1 { return 4; }
  if binary_length(0) != 1 { return 5; }
  if binary_length(1) != 1 { return 6; }
  if binary_length(7) != 3 { return 7; }
  if binary_length(8) != 4 { return 8; }
  return 0;
}
