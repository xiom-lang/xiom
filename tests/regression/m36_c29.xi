// M36-C29: Combined mega test 2 -- modules, methods, enums, pointers, Result, strings, type aliases
type Score = { points: Int; id: Int; } derive[Eq]
fn Score.read(self) -> Int { return self.points; }
enum Status { Active, Paused, Stopped }
fn Status.to_int(self) -> Int { match self { Active => 1, Paused => 2, Stopped => 3 } }
fn ptr_roundtrip() -> Int {
  var val: Int = 77;
  var p: *Int;
  unsafe { p = &val as *Int; }
  var v: Int;
  unsafe { v = *p; }
  return v;
}
module helpers {
  pub fn clamp(x: Int, lo: Int, hi: Int) -> Int {
    if x < lo { return lo; }
    if x > hi { return hi; }
    return x;
  }
  pub fn is_even(n: Int) -> Bool { return n % 2 == 0; }
  pub fn greet() -> Str { return "hello world"; }
}
use helpers.clamp;
use helpers.is_even;
use helpers.greet;
fn compose_result(ok_val: Int) -> Result[Int, Str] {
  if ok_val > 0 { return Ok(ok_val); }
  return Err("bad value");
}
fn main() -> Int {
  var sc = Score{ points: 100; id: 1; };
  if sc.read() != 100 { return 1; }
  if Status.to_int(Active) != 1 { return 2; }
  if Status.to_int(Paused) != 2 { return 3; }
  if Status.to_int(Stopped) != 3 { return 4; }
  if ptr_roundtrip() != 77 { return 5; }
  if clamp(50, 0, 100) != 50 { return 6; }
  if clamp(-10, 0, 100) != 0 { return 7; }
  if clamp(200, 0, 100) != 100 { return 8; }
  if !is_even(4) { return 9; }
  if is_even(3) { return 10; }
  if greet() != "hello world" { return 11; }
  match compose_result(5) { Ok(v) => { if v != 5 { return 12; } } Err(_) => { return 13; } }
  match compose_result(0) { Ok(_) => { return 14; } Err(e) => { if e != "bad value" { return 15; } } }
  match compose_result(-1) { Ok(_) => { return 16; } Err(_) => {} }
  return 0;
}
