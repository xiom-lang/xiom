// m87 (R45): tuple element types must use XIOM names, not LLVM widths.
//   - comparisons / logical ops are Bool (i64 at the ABI),
//   - `x as UInt16` uses the CAST TARGET, not i16 -> "Int16",
//   - tracked Bool locals keep Bool.
// Pre-fix the literal path named these elements "Int"/"Int16", building
// Tuple__Int__Int / Tuple__Int16__Int16 while the signature said
// Tuple__Int__Bool / Tuple__UInt16__UInt16 -> clang rejects the return
// (stdlib single-param sweep: overflowing_neg, code_point_to_utf16).
module m87_tuple_element_types

fn gt(a: Int) -> (Int, Bool) {
  (a, a > 0)
}

fn armed(a: Int) -> (Int, Bool) {
  var ok: Bool = true;
  if a == 0 {
    var fl: Bool = false;
    return (a, fl);
  };
  (a + 1, ok)
}

fn split16(v: Int) -> (UInt16, UInt16) {
  var hi = (v >> 16) & 0xFFFF;
  var lo = v & 0xFFFF;
  return (hi as UInt16, lo as UInt16);
}

fn tagged(label: Str, ok: Bool) -> (Str, Bool) {
  (label, ok)
}

// Monomorphised generic: the element `a: T` must name the tuple after the
// CONCRETE type (Bool), not the unresolved parameter or its i64 width.
fn pair[T](a: T, b: Float64) -> (T, Float64) {
  (a, b)
}

fn main() -> Int {
  let g = gt(5);
  if !g.1 { return 1; }
  if g.0 != 5 { return 2; }

  let z = armed(0);
  if z.1 { return 3; }
  if z.0 != 0 { return 4; }
  let n = armed(9);
  if !n.1 { return 5; }
  if n.0 != 10 { return 6; }

  let s = split16(0x0102_0304);
  if s.0 != 0x0102 { return 7; }
  if s.1 != 0x0304 { return 8; }

  let t = tagged("x", true);
  if !t.1 { return 9; }
  if t.0.len() != 1 { return 10; }

  let pb = pair(true, 1.5);
  if !pb.0 { return 11; }
  if pb.1 != 1.5 { return 12; }

  return 0;
}
