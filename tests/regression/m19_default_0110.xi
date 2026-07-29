module regression.m19_default_0110

interface Wrapper {
  fn wrap(&self) -> Option[Int] { var v = get(); if v >= 0 { return Some(v); } return None; }
  fn get(&self) -> Int;
}

type Val = { n: Int; }

fn Val.get(&self) -> Int { return n; }

fn main() -> Int {
  var pos: Val = Val{ n: 99 };
  var w = pos.wrap();
  if w == Some(99) { return 0; }
  return 1;
}
