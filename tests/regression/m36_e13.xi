// M36-E13: Mixed case identifiers — camelCase, PascalCase, snake_case
fn myFunc() -> Int { return 10; }
type MyType = { MyField: Int; }
fn main() -> Int {
  var my_var = myFunc();
  var mt = MyType{ MyField: my_var };
  if mt.MyField != 10 { return 1; }
  return 0;
}
