// m88 (R46): Box[T] with ONE field -- shares the leaf with m88.hard's Box.
module m88.generics

pub type Box[T] = {
  value: T;
}

pub fn Box.new[T](v: T) -> Box[T] {
  return Box[T]{ value: v };
}

pub fn Box.value_of[T]() -> T {
  return value;
}

// Non-generic wrappers keep instantiation module-local (the lock targets the
// same-leaf layout collision, not cross-module generic method mono).
pub fn make_plain() -> Box[Int] {
  return Box.new[Int](7);
}

pub fn plain_value(b: &Box[Int]) -> Int {
  return b.value_of[Int]();
}
