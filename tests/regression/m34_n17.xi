// M34-N17: Derive on type with all primitives -- struct containing Int, Float64, Bool, Char, Str
type AllTypes = { i: Int; f: Float64; b: Bool; c: Char; s: Str; } derive[Eq, Clone]
fn main() -> Int {
  var a = AllTypes{ i: 42; f: 3.14; b: true; c: 'X'; s: "hello"; };
  var b = a.clone();
  var c = AllTypes{ i: 99; f: 2.71; b: false; c: 'Y'; s: "world"; };
  if a == b && a != c { return 0; }
  return 1;
}
