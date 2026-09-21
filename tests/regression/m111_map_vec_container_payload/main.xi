// m111 (L5-40): Map[Int, Vec[Str]] -- a container PAYLOAD inside a generic
// container. Before the fix three sites disagreed on V:
//   Map.new    -> _Int_Int        (values Vec allocated with 8-byte slots)
//   Map.insert -> _Int_Vec        (stored a 32-byte %struct.Vec header)
//   Map.get    -> _Int_Vec_Str_   (read the slot as an 8-byte heap handle)
// and the match binder resolved field 1 through the ERASED Option
// registration (i64) instead of the concrete %struct.Option__Vec_Str_
// (inline %struct.Vec), so `g.len()` printed a pointer fragment.
module m111.main

use xiom.io;

fn main() -> Int {
  let m: Map[Int, Vec[Str]] = Map[Int, Vec[Str]].new();
  var v: Vec[Str] = Vec[Str].new();
  v.push("a");
  v.push("b");
  m.insert(25, v);
  if m.len() != 1 { return 1; }
  let g = match m.get(&25) { Some(x) => x, None => Vec[Str].new() };
  if g.len() != 2 { return 2; }
  if g.get(0).unwrap() != "a" { return 3; }
  if g.get(1).unwrap() != "b" { return 4; }
  io.println("ok");
  return 0;
}
