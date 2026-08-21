// M35-T02: Int in every context -- var, param, return, struct, enum, array, generic, if, while, match, operators
type IntBox = { value: Int; }
enum IntOption { Some(v: Int), None }
fn int_add(a: Int, b: Int) -> Int { return a + b; }
fn int_mul(a: Int, b: Int) -> Int { return a * b; }
fn int_if(flag: Int) -> Int { if flag > 0 { return flag; } return 0; }
fn int_while_sum(n: Int) -> Int { var i = 0; var s = 0; while i < n { s = s + i; i = i + 1; } return s; }
fn int_match(v: Int) -> Int { match v { 0 => 100, 1 => 200, _ => 0 } }
fn int_generic[T](x: T) -> T { return x; }
fn main() -> Int {
  var a: Int = 42; var b = 10;
  var s = int_add(a, b); if s != 52 { return 1; }
  var m = int_mul(a, 2); if m != 84 { return 2; }
  if int_if(5) != 5 { return 3; }
  if int_if(-1) != 0 { return 4; }
  if int_while_sum(10) != 45 { return 5; }
  if int_match(0) != 100 { return 6; }
  if int_match(1) != 200 { return 7; }
  if int_match(99) != 0 { return 8; }
  var box = IntBox{ value: 77 }; if box.value != 77 { return 9; }
  var arr = [1, 2, 3]; if arr[1] != 2 { return 10; }
  var g = int_generic(88); if g != 88 { return 11; }
  var e = IntOption.Some(55); match e { IntOption.Some(v) => if v != 55 { return 12; } IntOption.None => return 13; }
  var neg: Int = 0 - 10; if neg != -10 { return 14; }
  var mod_val: Int = 17 % 5; if mod_val != 2 { return 15; }
  return 0;
}

