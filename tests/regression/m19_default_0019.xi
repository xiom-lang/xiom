module regression.m19_default_0019

interface Wrappable {
  fn wrap(&self) -> Option[Int] { return Some(value()); }
  fn value(&self) -> Int;
}

type Holder = { x: Int; }

fn Holder.value(&self) -> Int { return x; }

fn main() -> Int {
  var h: Holder = Holder{ x: 7 };
  var result = h.wrap();
  match result {
    Some(v) => if v == 7 { return 0; },
    None => return 2
  }
  return 1;
}
