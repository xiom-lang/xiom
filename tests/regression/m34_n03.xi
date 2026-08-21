// M34-N03: Enum with derive[Eq] -- enum variant equality
enum Color { Red, Green, Blue } derive[Eq]
fn main() -> Int {
  var a = Color.Red;
  var b = Color.Red;
  var c = Color.Blue;
  if a == b && a != c { return 0; }
  return 1;
}
