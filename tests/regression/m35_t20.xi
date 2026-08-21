// M35-T20: Int exhaustive -- every context with all integer subtypes
fn int_add(a: Int, b: Int) -> Int { return a + b; }
fn int_sub(a: Int, b: Int) -> Int { return a - b; }
fn int_mul(a: Int, b: Int) -> Int { return a * b; }
fn int_div(a: Int, b: Int) -> Int { return a / b; }
fn int_mod(a: Int, b: Int) -> Int { return a % b; }
fn int_neg(a: Int) -> Int { return 0 - a; }
fn int_cmp(a: Int, b: Int) -> Bool { return a > b; }
type IntPair = { a: Int; b: Int; }
enum IntEnum { Val(v: Int), Zero, Neg(v: Int) }
fn int_match_enum(e: IntEnum) -> Int { match e { IntEnum.Val(v) => v, IntEnum.Zero => 0, IntEnum.Neg(v) => v } }
fn int_array_sum(arr: Vec[Int], n: Int) -> Int { var i: Int = 0; var s: Int = 0; while i < n { s = s + arr[i]; i = i + 1; } return s; }
fn main() -> Int {
  var a: Int = 42; var b: Int = 10;
  if int_add(a, b) != 52 { return 1; }
  if int_sub(a, b) != 32 { return 2; }
  if int_mul(a, 2) != 84 { return 3; }
  if int_div(a, 2) != 21 { return 4; }
  if int_mod(17, 5) != 2 { return 5; }
  if int_neg(5) != -5 { return 6; }
  if !int_cmp(a, b) { return 7; }
  if int_cmp(b, a) { return 8; }
  var ip: IntPair = IntPair{ a: 10; b: 20; };
  if ip.a + ip.b != 30 { return 9; }
  if int_match_enum(IntEnum.Val(55)) != 55 { return 10; }
  if int_match_enum(IntEnum.Zero) != 0 { return 11; }
  if int_match_enum(IntEnum.Neg(-7)) != -7 { return 12; }
  var arr: Vec[Int] = [1, 2, 3, 4, 5];
  if int_array_sum(arr, 5) != 15 { return 13; }
  var i: Int = 0; var s: Int = 0;
  while i < 10 { s = s + i; i = i + 1; }
  if s != 45 { return 14; }
  if (1 + 2) * 3 != 9 { return 15; }
  return 0;
}

