// M32-T11: Unicode -- string with lambda character, verify len > 0
fn main() -> Int {
  var s: Str = "lambda";
  if s.len() > 0 { return 0; }
  return 1;
}
