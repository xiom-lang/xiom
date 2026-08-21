// M35-T11: Enum with all variant types -- unit, Int payload, Float64 payload, Bool payload, Str payload, struct payload
type Point = { x: Int; y: Int; }
enum MixedEnum {
  A,
  B(i: Int),
  C(f: Float64),
  D(b: Bool),
  E(s: Str),
  F(p: Point),
  G(i: Int, f: Float64),
  H(i: Int, s: Str, b: Bool),
}
fn variant_check(e: MixedEnum) -> Int {
  match e {
    MixedEnum.A => 0,
    MixedEnum.B(i) => i,
    MixedEnum.C(f) => f as Int,
    MixedEnum.D(b) => if b { 100 } else { -100 },
    MixedEnum.E(s) => s.len() as Int,
    MixedEnum.F(p) => p.x + p.y,
    MixedEnum.G(i, f) => i + f as Int,
    MixedEnum.H(i, s, b) => { var r = i + s.len() as Int; if b { r = r + 100; } r },
  }
}
fn match_return_str(e: MixedEnum) -> Str {
  match e { MixedEnum.E(s) => s, _ => "other" }
}
fn match_return_bool(e: MixedEnum) -> Bool {
  match e { MixedEnum.D(b) => b, _ => false }
}
fn main() -> Int {
  if variant_check(MixedEnum.A) != 0 { return 1; }
  if variant_check(MixedEnum.B(42)) != 42 { return 2; }
  if variant_check(MixedEnum.C(3.0)) != 3 { return 3; }
  if variant_check(MixedEnum.D(true)) != 100 { return 4; }
  if variant_check(MixedEnum.D(false)) != -100 { return 5; }
  if variant_check(MixedEnum.E("hi")) != 2 { return 6; }
  if variant_check(MixedEnum.F(Point{ x: 3; y: 4; })) != 7 { return 7; }
  if variant_check(MixedEnum.G(10, 2.0)) != 12 { return 8; }
  var s = match_return_str(MixedEnum.E("hello")); if s != "hello" { return 9; }
  if !match_return_bool(MixedEnum.D(true)) { return 10; }
  return 0;
}

