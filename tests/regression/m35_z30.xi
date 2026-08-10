// M35-Z30: ALL_FEATURES: struct+enum+generic+match+while+if+contract+invariant+derive+impl+module+closure+Option+Result+array+pointer+unsafe+cast+compound_assign+const+type_alias+recursion
const THRESHOLD: Int = 10;
type Register = { id: Int; count: Int; invariant: count >= 0; } derive[Eq]
enum Event { Tick, Reset, Overflow, Underflow }
fn Register.zero() -> Register { return Register{ id: 0; count: 0; }; }
fn Register.inc(self) -> Register
  ensures: result.count >= self.count
{ var r = self; r.count += 1; return r; }
interface Countable { fn incr(self) -> Register; fn value(self) -> Int; }
impl Countable for Register {
  fn incr(self) -> Register { return self.inc(); }
  fn value(self) -> Int { return self.count; }
}
fn handle_event[T](r: Register, ev: Event) -> Int
  requires: r.count >= 0
{
  match ev {
    Tick => {
      var r2 = r.inc();
      return r2.count;
    }
    Reset => 0,
    Overflow => { if r.count > 1000 { return -1; } return r.count; }
    Underflow => { if r.count == 0 { return -2; } return r.count - 1; }
  }
}
fn factorial(n: Int) -> Int {
  if n <= 1 { return 1; }
  return n * factorial(n - 1);
}
fn check_magic() -> *Int {
  var p: *Int;
  unsafe { p = THRESHOLD as *Int; }
  return p;
}
module core {
  pub fn event(r: Register, e: Event) -> Int { return handle_event(r, e); }
  pub fn fact(n: Int) -> Int { return factorial(n); }
  pub fn magic_ptr() -> *Int {
    var p: *Int = check_magic();
    unsafe { return p; }
  }
}
use core.event;
use core.fact;
use core.magic_ptr;
fn main() -> Int {
  var r2 = Register.zero();
  r2.id = 2;
  r2.count = 5;
  var c1 = event(r2, Event.Underflow);
  var c2 = event(r2, Event.Overflow);
  var f = fact(5);
  var p = magic_ptr();
  var chk = 0;
  if c1 == 4 { chk += 1; }
  if c2 == 5 { chk += 1; }
  if f == 120 { chk += 1; }
  if p == unsafe { THRESHOLD as *Int } { chk += 1; }
  var arr = [1, 3, 5];
  var i = 0; var s = 0;
  while i < 3 { s += arr[i]; i += 1; }
  if s == 9 { chk += 1; }
  var r1 = Register.zero();
  r1.id = 1;
  r1 = r1.inc();
  r1 = r1.inc();
  if r1.count == 2 { chk += 1; }
  if chk == 6 { return 0; }
  return 1;
}
