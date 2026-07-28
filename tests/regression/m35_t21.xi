// M35-T21: Float64 exhaustive — every context: arith, cmp, cast, struct, enum, array, while
fn fadd(a: Float64, b: Float64) -> Float64 { return a + b; }
fn fsub(a: Float64, b: Float64) -> Float64 { return a - b; }
fn fmul(a: Float64, b: Float64) -> Float64 { return a * b; }
fn fdiv(a: Float64, b: Float64) -> Float64 { return a / b; }
fn fneg(a: Float64) -> Float64 { return -a; }
fn fcmp(a: Float64, b: Float64) -> Bool { return a > b; }
fn fcast(a: Int) -> Float64 { return a as Float64; }
type FloatPair = { a: Float64; b: Float64; }
enum FloatEnum { Val(v: Float64), Zero, Half }
fn float_match(e: FloatEnum) -> Float64 { match e { FloatEnum.Val(v) => v, FloatEnum.Zero => 0.0, FloatEnum.Half => 0.5 } }
fn float_while_acc(n: Int) -> Float64 { var i = 0; var acc: Float64 = 0.0; while i < n { acc = acc + 0.25; i = i + 1; } return acc; }
fn main() -> Int {
  var a: Float64 = 10.0; var b: Float64 = 4.0;
  if fadd(a, b) != 14.0 { return 1; }
  if fsub(a, b) != 6.0 { return 2; }
  if fmul(a, b) != 40.0 { return 3; }
  if fdiv(a, b) != 2.5 { return 4; }
  if fneg(1.0) != -1.0 { return 5; }
  if !fcmp(a, b) { return 6; }
  if fcmp(b, a) { return 7; }
  if fcast(42) != 42.0 { return 8; }
  var fp = FloatPair{ a: 1.5; b: 2.5; }; if fadd(fp.a, fp.b) != 4.0 { return 9; }
  if float_match(FloatEnum.Val(3.14)) != 3.14 { return 10; }
  if float_match(FloatEnum.Zero) != 0.0 { return 11; }
  if float_match(FloatEnum.Half) != 0.5 { return 12; }
  var acc = float_while_acc(4); if acc != 1.0 { return 13; }
  var arr = [1.1, 2.2, 3.3]; var sum: Float64 = 0.0; var i = 0; while i < 3 { sum = sum + arr[i]; i = i + 1; } if sum != 6.6 { return 14; }
  if a > 0.0 && b < 10.0 && fadd(1.0, 2.0) == 3.0 { } else { return 15; }
  return 0;
}

