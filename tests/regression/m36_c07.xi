// M36-C07: Every method pattern with every type — method on struct for Int, Bool, Float64, Str
type Counter = { val: Int; }
fn Counter.new() -> Counter { return Counter{ val: 0; }; }
fn Counter.inc(self) -> Counter { var r = self; r.val = r.val + 1; return r; }
fn Counter.add(self, n: Int) -> Counter { var r = self; r.val = r.val + n; return r; }
fn Counter.value(self) -> Int { return self.val; }
type Flag = { state: Bool; }
fn Flag.new() -> Flag { return Flag{ state: false; }; }
fn Flag.toggle(self) -> Flag { var r = self; r.state = !r.state; return r; }
fn Flag.is_set(self) -> Bool { return self.state; }
type Measure = { amount: Float64; }
fn Measure.new(v: Float64) -> Measure { return Measure{ amount: v; }; }
fn Measure.double(self) -> Measure { var r = self; r.amount = r.amount * 2.0; return r; }
fn Measure.get(self) -> Float64 { return self.amount; }
type Label = { text: Str; }
fn Label.make(s: Str) -> Label { return Label{ text: s; }; }
fn Label.len(self) -> Int { return self.text.len(); }
type BoxInt = { data: Int; }
fn BoxInt.new(v: Int) -> BoxInt { return BoxInt{ data: v; }; }
fn BoxInt.get(self) -> Int { return self.data; }
fn main() -> Int {
  var c = Counter.new();
  c = c.inc();
  c = c.add(5);
  if c.value() != 6 { return 1; }
  c = c.inc();
  if c.value() != 7 { return 2; }
  var f = Flag.new();
  f = f.toggle();
  if !f.is_set() { return 3; }
  f = f.toggle();
  if f.is_set() { return 4; }
  var m = Measure.new(3.5);
  m = m.double();
  if m.get() < 6.99 || m.get() > 7.01 { return 5; }
  m = Measure.new(0.0);
  m = m.double();
  if m.get() != 0.0 { return 6; }
  var lbl = Label.make("test");
  if lbl.len() != 4 { return 7; }
  var empty = Label.make("");
  if empty.len() != 0 { return 8; }
  var bi = BoxInt.new(42);
  if bi.get() != 42 { return 9; }
  return 0;
}
