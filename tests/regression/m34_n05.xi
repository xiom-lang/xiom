// M34-N05: Struct with derive[Clone] -- struct clone
type Data = { val: Int; name: Str; } derive[Clone]
fn main() -> Int {
  var a = Data{ val: 42; name: "test"; };
  var b = a.clone();
  if b.val == 42 && b.name == "test" { return 0; }
  return 1;
}
