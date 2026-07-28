// M35-M05: integer power function (iterative)
fn pow_int(base: Int, exp: Int) -> Int {
  if exp < 0 { return 0; }
  var result: Int = 1;
  var i: Int = 0;
  while i < exp {
    result = result * base;
    i = i + 1;
  }
  return result;
}
fn main() -> Int {
  if pow_int(2, 0) == 1 && pow_int(2, 3) == 8 && pow_int(3, 4) == 81 && pow_int(5, 2) == 25 && pow_int(7, 1) == 7 { return 0; }
  return 1;
}
