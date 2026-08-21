// M32-T13: String in struct -- struct with Str field
type Person = { name: Str; age: Int; }
fn main() -> Int {
  var p = Person{ name: "Alice"; age: 30; };
  if p.name == "Alice" && p.age == 30 { return 0; }
  return 1;
}
