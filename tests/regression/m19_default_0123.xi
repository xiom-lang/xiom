module regression.m19_default_0123

interface Predicates {
  fn is_pos(&self) -> Bool { return val() > 0; }
  fn is_even(&self) -> Bool { return val() % 2 == 0; }
  fn is_valid(&self) -> Bool { return is_pos() && is_even(); }
  fn val(&self) -> Int;
}

type Num = { n: Int; }

fn Num.val(&self) -> Int { return n; }

fn main() -> Int {
  var v: Num = Num{ n: 8 };
  if v.val() != 8 { return 1; }
  if v.is_pos() != true { return 2; }
  if v.is_even() != true { return 3; }
  if v.is_valid() != true { return 4; }
  return 0;
}
