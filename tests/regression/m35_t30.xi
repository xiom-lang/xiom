// M35-T30: Mega stress — types, generics, impl, variant
type All = { b: Bool; i: Int; s: Str; }
enum Variant { Unit, IntVal(v: Int), StrVal(v: Str) }
interface Score { fn score(self) -> Int; }
impl Score for All {
  fn score(self) -> Int {
    var s: Int = 0;
    if self.b { s = s + 1; }
    s = s + self.i;
    s = s + self.s.len() as Int;
    return s;
  }
}
fn variant_to_int(v: Variant) -> Int {
  match v {
    Variant.Unit => 0,
    Variant.IntVal(i) => i,
    Variant.StrVal(s) => s.len() as Int,
  }
}
fn main() -> Int {
  var a: All = All{ b: true; i: 10; s: "ok"; };
  var sc: Int = a.score();
  if sc != 13 { return 1; }
  if variant_to_int(Variant.Unit) != 0 { return 2; }
  if variant_to_int(Variant.IntVal(42)) != 42 { return 3; }
  if variant_to_int(Variant.StrVal("hi")) != 2 { return 4; }
  return 0;
}

