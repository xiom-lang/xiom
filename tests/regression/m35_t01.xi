// M35-T01: Bool in every context -- var, param, return, struct, enum, array, generic, if, while, match
fn negate(b: Bool) -> Bool { return !b; }
fn and_both(a: Bool, b: Bool) -> Bool { return a && b; }
fn or_either(a: Bool, b: Bool) -> Bool { return a || b; }
fn bool_if_ret(flag: Bool) -> Int { if flag { return 0; } return 1; }
fn bool_while_count(limit: Bool, n: Int) -> Int { var i = 0; var c = 0; while i < n { c = c + 1; i = i + 1; } return c; }
type BoolBox = { val: Bool; }
enum BoolResult { Yes, No, Maybe(b: Bool) }
fn bool_match(b: Bool) -> Int { match b { true => 0, false => 1 } }
fn generic_eq[T](a: T, b: T) -> Bool { return a == b; }
fn main() -> Int {
  var b1: Bool = true; var b2 = false;
  if negate(b1) { return 1; }
  if !negate(b2) { return 2; }
  if !and_both(true, true) { return 3; }
  if or_either(false, false) { return 4; }
  if bool_if_ret(true) != 0 { return 5; }
  if bool_if_ret(false) != 1 { return 6; }
  if bool_match(true) != 0 { return 7; }
  if bool_match(false) != 1 { return 8; }
  var bb = BoolBox{ val: true }; if !bb.val { return 9; }
  var e = BoolResult.Maybe(true); match e { BoolResult.Maybe(b) => if !b { return 10; } _ => 0 }
  var r = bool_while_count(true, 3); if r != 3 { return 11; }
  var arr = [true, false, true]; if arr[0] != true { return 12; } if arr[1] != false { return 13; }
  if !generic_eq(true, true) { return 14; }
  return 0;
}

