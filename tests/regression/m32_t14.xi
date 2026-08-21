// M32-T14: String in array -- an array of strings
type Entry = { key: Str; value: Str; }
fn main() -> Int {
  var a = Entry{ key: "name"; value: "Kilo"; };
  var b = Entry{ key: "type"; value: "AI"; };
  if a.key == "name" && a.value == "Kilo" && b.key == "type" && b.value == "AI" { return 0; }
  return 1;
}
