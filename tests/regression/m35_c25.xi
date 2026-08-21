// M35-C25: match with wildcard -- catch-all pattern _ for remaining cases
enum Signal { High, Low, Off, Unknown }
fn respond(s: Signal) -> Int {
  match s {
    High => 1,
    Low => 0,
    _ => -1,
  }
}
fn label(n: Int) -> Str {
  match n {
    0 => "zero",
    1 => "one",
    2 => "two",
    _ => "many",
  }
}
fn main() -> Int {
  if respond(Signal.High) != 1 { return 1; }
  if respond(Signal.Low) != 0 { return 2; }
  if respond(Signal.Off) != -1 { return 3; }
  if respond(Signal.Unknown) != -1 { return 4; }
  var r: Int = match 999 { 0 => 0, 1 => 1, _ => 42 };
  if r != 42 { return 5; }
  var r2: Int = match 1 { 0 => 0, 1 => 99, _ => 0 };
  if r2 != 99 { return 6; }
  return 0;
}
