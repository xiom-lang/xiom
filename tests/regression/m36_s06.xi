// M36-S06: Type checker — subtype and assignability checking
type TypeSig = { id: Int; flags: Int; base: Int; }
fn is_subtype(sub: TypeSig, sup: TypeSig) -> Bool {
  if sub.id == sup.id { return true; }
  if sub.base == sup.id { return true; }
  return false;
}
fn is_assignable(src: TypeSig, dst: TypeSig) -> Bool {
  if src.id == dst.id { return true; }
  if src.id == 0 && dst.id != 3 { return true; }
  return false;
}
fn is_numeric(sub: TypeSig) -> Bool {
  return sub.id >= 1 && sub.id <= 3;
}
fn can_widen(src: TypeSig, dst: TypeSig) -> Bool {
  if !is_numeric(src) || !is_numeric(dst) { return false; }
  return src.flags <= dst.flags;
}
fn main() -> Int {
  var int32 = TypeSig{ id: 1; flags: 4; base: 0; };
  var int64 = TypeSig{ id: 2; flags: 8; base: 0; };
  var float64 = TypeSig{ id: 3; flags: 8; base: 0; };
  var unknown = TypeSig{ id: 0; flags: 0; base: 0; };
  var child = TypeSig{ id: 10; flags: 0; base: 1; };
  if !is_subtype(int32, int32) { return 1; }
  if is_subtype(int32, int64) { return 2; }
  if !is_subtype(child, int32) { return 3; }
  if !is_assignable(int32, int32) { return 4; }
  if is_assignable(int32, float64) { return 5; }
  if !is_assignable(unknown, int32) { return 6; }
  if !can_widen(int32, int64) { return 7; }
  if !can_widen(int32, float64) { return 8; }
  if can_widen(int64, int32) { return 9; }
  return 0;
}
