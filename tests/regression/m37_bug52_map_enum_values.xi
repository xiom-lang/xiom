// BUG 52 regression: Map[Str, T] where T is an enum WITH payload fields.
// Four coordinated codegen fixes: enum-ctor generic args, local-receiver
// type-arg inference, mono field-Vec element recording (memcpy reads), and
// struct/enum index-write memcpy (duplicate-key update path).
module m37_bug52_map_enum_values
use xiom.collections;

type MyVal = enum {
  Num(value: Int),
  Text(value: Str),
}

fn main() -> Int {
  var m = Map[Str, MyVal].new();
  m.insert("b", MyVal.Text("x"));
  m.insert("n", MyVal.Num(42));
  m.insert("b", MyVal.Num(7));
  if m.len() != 2 { return 1; }
  match m.get("b") {
    Some(val) => {
      match val {
        Num(n) => {
          if n != 7 { return 2; }
        }
        Text(_) => { return 3; }
      }
    }
    None => { return 4; }
  }
  match m.get("n") {
    Some(val) => {
      match val {
        Num(n) => {
          if n != 42 { return 5; }
        }
        Text(_) => { return 6; }
      }
    }
    None => { return 7; }
  }
  let r = m.remove("b");
  if !r.is_some { return 8; }
  if m.len() != 1 { return 9; }
  return 0;
}
