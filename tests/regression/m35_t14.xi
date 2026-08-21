// M35-T14: Const for each primitive -- Bool, Int, Float64, Char, Str
const C_BOOL: Bool = true;
const C_INT: Int = 42;
const C_FLOAT: Float64 = 3.14;
const C_CHAR: Char = 'X';
const C_NEG: Int = -10;
const C_ZERO: Int = 0;
fn const_arith() -> Int { return C_INT + C_NEG + C_ZERO; }
fn const_compare() -> Bool { return C_INT > C_ZERO && C_INT == 42 && C_FLOAT > 0.0; }
fn main() -> Int {
  if !C_BOOL { return 1; }
  if C_INT != 42 { return 2; }
  if C_FLOAT != 3.14 { return 3; }
  if C_CHAR != 'X' { return 4; }
  if C_NEG != -10 { return 5; }
  if C_ZERO != 0 { return 6; }
  var ari: Int = const_arith();
  if ari != 32 { return 7; }
  if !const_compare() { return 8; }
  return 0;
}

