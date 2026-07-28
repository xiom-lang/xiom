// M35-T19: Bool exhaustive — every context
type BoolBox = { val: Bool; }
enum BoolResult { Yes, No, Flip(b: Bool) }
fn not_fn(b: Bool) -> Bool { return !b; }
fn and_fn(a: Bool, b: Bool) -> Bool { return a && b; }
fn or_fn(a: Bool, b: Bool) -> Bool { return a || b; }
fn xor_fn(a: Bool, b: Bool) -> Bool { return (a || b) && !(a && b); }
fn bool_to_int(b: Bool) -> Int { if b { return 1; } return 0; }
fn int_to_bool(i: Int) -> Bool { return i != 0; }
fn main() -> Int {
  var t: Bool = true; var f: Bool = false;
  if not_fn(t) { return 1; }
  if !not_fn(f) { return 2; }
  if !and_fn(t, t) { return 3; }
  if and_fn(t, f) { return 4; }
  if !or_fn(f, t) { return 5; }
  if or_fn(f, f) { return 6; }
  if xor_fn(t, t) { return 7; }
  if !xor_fn(t, f) { return 8; }
  if bool_to_int(t) != 1 { return 9; }
  if bool_to_int(f) != 0 { return 10; }
  if !int_to_bool(1) { return 11; }
  if int_to_bool(0) { return 12; }
  var box: BoolBox = BoolBox{ val: true };
  if !box.val { return 13; }
  var er1: BoolResult = BoolResult.Flip(true);
  match er1 { BoolResult.Flip(b) => if !b { return 14; } _ => return 15; }
  var count: Int = 0; var i: Int = 0;
  while i < 3 { count = count + 1; i = i + 1; }
  if count != 3 { return 15; }
  return 0;
}

